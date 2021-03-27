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

    /// Json output
    #[structopt(short, long)]
    pub json: bool,

    /// Calculate daily change
    #[structopt(long)]
    pub dy: bool,

    /// Calculate weekly change
    #[structopt(long)]
    pub wk: bool,

    /// Calculate monthly change
    #[structopt(long)]
    pub mo: bool,

    /// Calculate yearly change
    #[structopt(long)]
    pub yr: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Cli = Cli::from_args();

    let tickers = join_all(
        args.tickers
            .iter()
            .cloned()
            .map(|ticker| task::spawn(get_ticker(ticker))),
    )
    .await
    .into_iter()
    .map(|res| -> Result<Ticker> { Ok(Ticker::try_from(res??)?) })
    .collect::<Result<Vec<_>>>();

    print(&args, tickers);
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

fn print(args: &Cli, ticker: Result<Vec<Ticker>>) {
    match ticker {
        Ok(t) => {
            if args.json {
                print_json(t);
            } else {
                t.into_iter().for_each(|x| print_ticker(args, x));
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn print_json(tickers: Vec<Ticker>) {
    match serde_json::to_string_pretty(&tickers) {
        Ok(t) => println!("{}", t),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn print_ticker(args: &Cli, ticker: Ticker) {
    let mut print = format!("{} {}", ticker.symbol, ticker.value);
    if args.dy {
        print.push_str(&format!(" {:.3}%", ticker.daily_change))
    }
    if args.wk {
        print.push_str(&format!(" {:.3}%", ticker.wk_change))
    }
    if args.mo {
        print.push_str(&format!(" {:.3}%", ticker.mo_change))
    }
    if args.yr {
        print.push_str(&format!(" {:.3}%", ticker.yr_change))
    }
    println!("{}", print);
}
