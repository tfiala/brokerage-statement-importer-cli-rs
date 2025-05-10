use std::path::PathBuf;

use anyhow::Result;
use mongodb::{Client, Database};
use rstest::fixture;
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

pub struct DbDesc {
    pub _client: Client,
    pub db: Database,
    pub _node: ContainerAsync<Mongo>,
}

impl DbDesc {
    pub async fn new(db_name: &str) -> Result<Self> {
        let node = Mongo::default().start().await?;
        let host_port = node.get_host_port_ipv4(27017).await?;

        let url = format!("mongodb://localhost:{}/", host_port);
        let client = mongodb::Client::with_uri_str(url).await?;
        let db = client.database(db_name);

        Ok(Self {
            _client: client,
            db,
            _node: node,
        })
    }
}

#[fixture]
pub async fn db_desc() -> Result<DbDesc> {
    let db_conn = DbDesc::new("test").await?;
    brokerage_db::initialize(&db_conn.db).await?;
    Ok(db_conn)
}

pub const IBKR_BROKERAGE_ACCOUNT_ID: &str = "U1234567";

#[fixture]
pub fn single_trade_flex() -> &'static str {
    r##"
    <FlexQueryResponse queryName="example-query" type="AF">
    <FlexStatements count="1">
    <FlexStatement accountId="U1234567" fromDate="2025-04-25" toDate="2025-04-25" period="LastBusinessDay" whenGenerated="2025-04-26;13:34:28 EDT">
    <AccountInformation accountId="U1234567" accountType="Individual" customerType="Individual" accountCapabilities="Portfolio Margin" tradingPermissions="Stocks,Options,Warrants,Forex,Futures,Crypto Currencies,Mutual Funds,Fully Paid Stock Loan" />
    <Trades>
        <Trade accountId="U1234567"
                currency="USD"
                symbol="ARGX"
                conid="276343981"
                listingExchange="NASDAQ"
                tradeID="7587063231"
                reportDate="2025-04-25"
                dateTime="2025-04-25;10:19:55 EDT"
                tradeDate="2025-04-25"
                transactionType="ExchTrade"
                exchange="BYX"
                quantity="1"
                tradePrice="606.57"
                tradeMoney="606.57"
                proceeds="-606.57"
                ibCommission="-1.000035"
                ibCommissionCurrency="USD"
                netCash="-607.570035"
                closePrice="614.76"
                openCloseIndicator="O"
                cost="607.570035"
                fifoPnlRealized="0"
                mtmPnl="8.19"
                origTradePrice="0"
                origTradeDate=""
                origTradeID=""
                origOrderID="0"
                origTransactionID="0"
                buySell="BUY"
                ibOrderID="4015030800"
                transactionID="32580112485"
                ibExecID="0000edae.680b59d1.01.01"
                orderTime="2025-04-25;10:19:55 EDT"
                openDateTime=""
                holdingPeriodDateTime=""
                whenRealized=""
                whenReopened=""
                orderType="LMT"
                accruedInt="0"
                assetCategory="STK"
                brokerageOrderID="002ce642.00014b44.680b0ed6.0001"
                orderReference=""
                isAPIOrder="N"
                initialInvestment="" />
    </Trades>
    </FlexStatement>
    </FlexStatements>
    </FlexQueryResponse>
    "##
}

#[fixture]
pub fn single_trade_flex_pathbuf() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("data")
        .join("ibkr_flex_single_trade.xml")
}
