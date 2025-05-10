mod fixtures;
#[path = "../src/importer/mod.rs"]
mod importer;

use std::path::PathBuf;

use anyhow::Result;
use brokerage_db::account::BrokerageAccount;
use fixtures::{DbDesc, db_desc, single_trade_flex, single_trade_flex_pathbuf};
use rstest::rstest;

#[test]
fn test_filter_unimported_files_with_no_previous_imports() {
    let paths = vec![
        PathBuf::from("test1.xml"),
        PathBuf::from("test2.xml"),
        PathBuf::from("test3.xml"),
    ];
    let filtered_paths = importer::filter_unimported_files(paths).unwrap();
    assert_eq!(filtered_paths.len(), 3);
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_import_string_ibkr_single_trade_execution(
    #[future] db_desc: Result<DbDesc>,
    single_trade_flex: &str,
) -> Result<()> {
    let db_desc = db_desc?;

    // Test the importer
    importer::import_ibkr_flex_content(&db_desc.db, "flex-statement.xml", single_trade_flex)
        .await?;

    // Verify the account was added.
    let brokerage_account = BrokerageAccount::find_by_brokerage_and_account_id(
        &db_desc.db,
        importer::IBKR_BROKERAGE_ID,
        fixtures::IBKR_BROKERAGE_ACCOUNT_ID,
    )
    .await?;
    assert!(
        brokerage_account.is_some(),
        "Brokerage account should exist"
    );

    let brokerage_account = brokerage_account.unwrap();
    assert_eq!(
        brokerage_account.get_account_id(),
        fixtures::IBKR_BROKERAGE_ACCOUNT_ID
    );
    assert_eq!(
        brokerage_account.get_brokerage_id(),
        importer::IBKR_BROKERAGE_ID
    );

    Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_import_file_ibkr_single_trade_execution(
    #[future] db_desc: Result<DbDesc>,
    single_trade_flex_pathbuf: PathBuf,
) -> Result<()> {
    let db_desc = db_desc?;

    // Test the importer
    importer::import_ibkr_flex_statement_files(&db_desc.db, vec![single_trade_flex_pathbuf])
        .await?;

    // Verify the account was added.
    let brokerage_account = BrokerageAccount::find_by_brokerage_and_account_id(
        &db_desc.db,
        importer::IBKR_BROKERAGE_ID,
        fixtures::IBKR_BROKERAGE_ACCOUNT_ID,
    )
    .await?;
    assert!(
        brokerage_account.is_some(),
        "Brokerage account should exist"
    );

    let brokerage_account = brokerage_account.unwrap();
    assert_eq!(
        brokerage_account.get_account_id(),
        fixtures::IBKR_BROKERAGE_ACCOUNT_ID
    );
    assert_eq!(
        brokerage_account.get_brokerage_id(),
        importer::IBKR_BROKERAGE_ID
    );

    Ok(())
}
