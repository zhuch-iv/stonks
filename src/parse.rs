use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Ticker {
    pub symbol: String,
    pub value: f64,
    pub daily_change: f64,
    pub wk_change: f64,
    pub mo_change: f64,
    pub yr_change: f64,
}

const ERROR_MSG: &str = "Failed to read response from server";

impl TryFrom<Response> for Ticker {
    type Error = anyhow::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let mut result = response
            .chart
            .result
            .with_context(|| ERROR_MSG)? //TODO
            .pop()
            .with_context(|| ERROR_MSG)?; //TODO

        let quote = result.indicators.quote.pop().with_context(|| ERROR_MSG)?; //TODO

        let open = quote.open;
        let close = quote.close;
        let value = result.meta.regular_market_price;
        Ok(Ticker {
            symbol: result.meta.symbol,
            value,
            daily_change: calc_change(value, open[open.len() - 1]),
            wk_change: calc_change(value, close[close.len() - 2]),
            mo_change: calc_change(value, close[close.len() - 5]),
            yr_change: calc_change(value, close[0]),
        })
    }
}

fn calc_change(now: f64, old: f64) -> f64 {
    ((now - old) / old) * 100.0
}

#[derive(Deserialize, Debug)]
pub struct Response {
    chart: Chart,
}

#[derive(Deserialize, Debug)]
struct Chart {
    result: Option<Vec<Res>>,
    error: Option<Error>, //TODO
}

#[derive(Deserialize, Debug)]
struct Res {
    meta: Meta,
    timestamp: Vec<u64>,
    events: Option<Events>,
    indicators: Indicators,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Meta {
    currency: String,
    symbol: String,
    exchange_name: String,
    instrument_type: String,
    first_trade_date: i64,
    exchange_timezone_name: String,
    regular_market_time: u64,
    regular_market_price: f64,
    data_granularity: String,
    valid_ranges: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Events {
    dividends: HashMap<String, Dividend>,
    splits: HashMap<String, Split>,
}

#[derive(Deserialize, Debug)]
struct Dividend {
    date: u64,
    amount: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Split {
    date: u64,
    numerator: u64,
    denominator: u64,
    split_ratio: String,
}

#[derive(Deserialize, Debug)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Vec<Adjclose>,
}

#[derive(Deserialize, Debug)]
struct Quote {
    open: Vec<f64>,
    volume: Vec<u64>,
    close: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct Adjclose {
    adjclose: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct Error {
    code: String,
    description: String,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use chrono::{DateTime, Utc};

    use crate::parse::*;

    #[test]
    fn parse_error_json() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/error.json");
        let json = std::fs::read_to_string(d).unwrap();

        let r: Response = serde_json::from_str(&json).unwrap();

        let error = r.chart.error.unwrap();

        assert_eq!("Not Found", &error.code);
        assert_eq!("No data found, symbol may be delisted", &error.description);
    }

    #[test]
    fn parse_mo_json() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/mo.json");
        let json = std::fs::read_to_string(d).unwrap();

        let r: Response = serde_json::from_str(&json).unwrap();

        let result = r.chart.result.unwrap();
        assert_eq!("USD", &result[0].meta.currency);
        assert_eq!("^GSPC", &result[0].meta.symbol);
        assert_eq!(3933.21f64, result[0].meta.regular_market_price);
        assert_eq!("1mo", &result[0].meta.data_granularity);
    }

    #[test]
    fn parse_wk_json() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/wk.json");
        let json = std::fs::read_to_string(d).unwrap();

        let r: Response = serde_json::from_str(&json).unwrap();

        let result = r.chart.result.unwrap();
        assert_eq!("USD", &result[0].meta.currency);
        assert_eq!("AAPL", &result[0].meta.symbol);
        assert_eq!(122.87f64, result[0].meta.regular_market_price);
        assert_eq!("1wk", &result[0].meta.data_granularity);
    }

    #[test]
    fn change_test() {
        assert_eq!(10.0, calc_change(110.0, 100.0));
        assert_eq!(-10.0, calc_change(90.0, 100.0));
        assert_eq!(-100.0, calc_change(0.0, 100.0));
    }
}
