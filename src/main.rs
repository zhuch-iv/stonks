use std::convert::TryFrom;

use anyhow::Result;
use chrono::Duration;
use futures::future::join_all;
use log::info;
use structopt::StructOpt;
use tokio::task;

use crate::parse::{Response, Ticker};

mod parse;

#[derive(StructOpt, Debug)]
struct Cli {
    pub tickers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::from_args();

    let vec: Vec<String> = vec!["AAPL".to_owned(), "^GSPC".to_owned()];
    let futures = vec
        .into_iter()
        .map(|ticker| task::spawn(get_ticker(ticker)))
        .collect::<Vec<_>>();

    for resp in join_all(futures).await {
        let ticker = Ticker::try_from(resp??)?;
        println!("{} {} {:.3}", ticker.symbol, ticker.value, ticker.mo_change)
    }

    Ok(())
}

fn calc_url(ticker: &str) -> reqwest::Url {
    let now = chrono::offset::Utc::now();
    let year_ago = now - Duration::days(365);

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?formatted=true\
        &lang=en-US&region=US&includeAdjustedClose=true&interval=1wk&period1={}\
        &period2={}&events=div%7Csplit&useYfid=true",
        ticker,
        year_ago.timestamp(),
        now.timestamp()
    );
    reqwest::Url::parse(&url).unwrap()
}

async fn get_ticker(ticker: String) -> Result<Response> {
    let response = reqwest::get(calc_url(&ticker)).await?;
    info!("Requesting ticker {}", &ticker);

    let resp: Response = response.json().await?;
    Ok(resp)
}
