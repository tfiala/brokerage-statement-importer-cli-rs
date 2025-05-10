use anyhow::Result;
use brokerage_statement_importer::ibkr_flex_statement_importer::IbkrFlexStatementImporter;
use brokerage_statement_importer::importer_registry::ImporterRegistry;
use clap::{Args, Parser, Subcommand};
use mongodb::{Client, ClientSession, Database};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "tdb")]
#[command(version, about = "trader database cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// The database URI to connect to. Defaults to DB_URI in .env file.
    #[arg(short, long, required = false)]
    db_uri: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Import brokerage statements into the db
    Import(ImportArgs),
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[command(subcommand)]
    command: ImportCommands,

    /// Use a transaction to wrap the import database operations. Requires MongoDB to be running in replica set mode. Default to false.
    #[arg(short, long, required = false)]
    with_transaction: Option<bool>,
}

#[derive(Subcommand, Debug)]
enum ImportCommands {
    /// Select brokerage statements via a regex (i.e. file glob) expression
    #[command(arg_required_else_help = true)]
    Regex { regex: String },

    /// Select brokerage statements via one or more filenames.
    #[command(arg_required_else_help = true)]
    Files {
        /// Statement filenames to import.
        #[arg(required = true)]
        path: Vec<PathBuf>,
    },
}

async fn maybe_start_transaction(
    client: &Client,
    with_transaction: Option<bool>,
) -> Result<Option<Arc<Mutex<ClientSession>>>> {
    if with_transaction.unwrap_or(false) {
        let mut session = client.start_session().await?;
        session.start_transaction().await?;
        Ok(Some(Arc::new(Mutex::new(session))))
    } else {
        Ok(None)
    }
}

async fn import_statement_files(
    importer_registry: &ImporterRegistry,
    db: &Database,
    session: Option<Arc<Mutex<ClientSession>>>,
    import_paths: Vec<PathBuf>,
) -> Result<()> {
    if import_paths.is_empty() {
        println!("all statements already imported");
        Ok(())
    } else {
        let result = importer_registry
            .import_statement_files(db, session.clone(), import_paths)
            .await;

        if result.is_err() {
            println!("failed to import files");
            if let Some(session) = session {
                let mut session = session.lock().await;
                session.abort_transaction().await?;
            }
            result
        } else {
            if let Some(session) = session {
                let mut session = session.lock().await;
                session.commit_transaction().await?;
            }
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the environment
    dotenvy::dotenv()?;

    let args = Cli::parse();

    let db_uri = match args.db_uri {
        Some(uri) => uri,
        None => env::var("DB_URI")
            .expect("DATABASE_URL not set in environment")
            .to_string(),
    };
    println!("Connecting to database at: {}", db_uri);

    // Connect to the database.
    let client = Client::with_uri_str(&db_uri)
        .await
        .expect("Failed to connect to database");
    println!("Connected to database");

    // Get the database.
    let db_name = env::var("DB_NAME")
        .expect("DB_NAME not set in environment")
        .to_string();
    let db = client.database(&db_name);
    println!("Using database: {}", db_name);

    // Setup the importer registry.
    let mut importer_registry = ImporterRegistry::new();
    importer_registry.register_importer(Box::new(IbkrFlexStatementImporter::new()));

    match args.command {
        Commands::Import(import_args) => {
            // Create a transaction.
            let session = maybe_start_transaction(&client, import_args.with_transaction)
                .await
                .expect("Failed to start transaction");

            match import_args.command {
                ImportCommands::Regex { regex } => {
                    let globbed_paths = glob::glob(regex.as_str())?
                        .filter_map(|entry| entry.ok())
                        .collect::<Vec<PathBuf>>();

                    // let import_paths = importer::filter_unimported_files(globbed_paths)?;
                    // TODO fix me
                    let mut import_paths = globbed_paths;
                    import_paths.sort();

                    import_statement_files(&importer_registry, &db, session, import_paths).await
                }

                ImportCommands::Files { path } => {
                    // let import_paths = importer::filter_unimported_files(path)?;
                    // TODO fix me
                    let mut import_paths = path.clone();
                    import_paths.sort();

                    import_statement_files(&importer_registry, &db, session, import_paths).await
                }
            }
        }
    }
}
