use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use crate::error::QuoteServerError;

#[derive(Debug, Serialize, Deserialize)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}


///1. Генерация данных. Создайте функцию, которая генерирует искусственные данные о ценах акций
/// (например, для тикеров "AAPL", "GOOGL", "TSLA").
/// Данные должны включать тикер, цену, объём и timestamp. Цены должны периодически меняться, например, случайным блужданием.
/// Пример структуры для генерации данных тикера:

#[derive(Debug)]
pub struct QuoteGenerator {
    quote_to_last_price: HashMap<String, f64>,
}
impl QuoteGenerator {

    pub fn new() -> Self {
        // panic in case we can't load data, unexpected
        Self::from_tickers_file("quote_server/stock_data/tickers.txt").expect("Cannot load stock_data")
    }

    pub fn from_tickers_file(path: &str) -> Result<QuoteGenerator, QuoteServerError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut quote_to_last_price: HashMap<String, f64> = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let ticker = line.trim();
            if ticker.is_empty() {
                continue;
            }

            let initial_price = match ticker {
                "AAPL" | "MSFT" | "GOOGL" | "AMZN" | "NVDA" | "META" | "TSLA" => (rand::random::<f64>() * 1000.0) ,
                _ => 100.00
            };
            println!("Price: {:?}", initial_price);
            quote_to_last_price.insert(String::from(ticker), initial_price);
        }

        Ok(QuoteGenerator{quote_to_last_price})
    }

    pub fn generate_quote(&mut self, ticker: &str) -> Option<StockQuote> {
        // ... логика изменения цены ...

        let volume = match ticker {
            "AAPL" | "MSFT" | "GOOGL" | "AMZN" | "NVDA" | "META" | "TSLA" => 1000 + (rand::random::<f64>() * 5000.0) as u32,
            _ => 100 + (rand::random::<f64>() * 1000.0) as u32,
        };

        todo!()
    }
}