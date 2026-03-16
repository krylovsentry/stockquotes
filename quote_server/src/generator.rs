use crate::error::QuoteServerError;
use quote_lib::StockQuote;
use rand::RngExt;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct QuoteGenerator {
    quote_to_last_price: HashMap<String, f64>,
}
impl QuoteGenerator {
    /// Создаёт генератор, загружая тикеры из стандартного файла.
    /// При ошибке чтения возвращает `QuoteServerError::Io`.
    pub fn new() -> Result<Self, QuoteServerError> {
        Self::from_tickers_file("quote_server/stock_data/tickers.txt")
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
                "AAPL" | "MSFT" | "GOOGL" | "AMZN" | "NVDA" | "META" | "TSLA" => {
                    rand::random::<f64>() * 1000.0
                }
                _ => 100.0,
            };
            quote_to_last_price.insert(String::from(ticker), initial_price);
        }

        Ok(QuoteGenerator {
            quote_to_last_price,
        })
    }

    pub fn generate_quote(&mut self, ticker: &str) -> StockQuote {
        let price_of_ticker = self
            .quote_to_last_price
            .entry(String::from(ticker))
            .or_insert(100.0);

        let mut rng = rand::rng();
        let delta = rng.random_range(-0.05..=0.05);
        let mut new_price = *price_of_ticker * (1.0 + delta);

        if new_price < 10.0 {
            new_price = 10.0;
        }

        let volume = match ticker {
            "AAPL" | "MSFT" | "GOOGL" | "AMZN" | "NVDA" | "META" | "TSLA" => {
                1000 + (rand::random::<f64>() * 5000.0) as u32
            }
            _ => 100 + (rand::random::<f64>() * 1000.0) as u32,
        };

        let ticker_name = String::from(ticker);
        StockQuote {
            ticker: ticker_name,
            price: new_price,
            volume,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn get_quotes(&mut self) -> Vec<String> {
        Vec::from_iter(self.quote_to_last_price.keys().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteGenerator;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    fn write_tickers(content: &str, name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("quote_server_test_{}.txt", name));
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.sync_all().unwrap();
        p
    }

    #[test]
    fn from_tickers_file_loads_tickers() {
        let path = write_tickers("AAPL\nTSLA\nGOOGL\n", "loads");
        let quote_gen = QuoteGenerator::from_tickers_file(path.to_str().unwrap()).unwrap();
        assert_eq!(quote_gen.quote_to_last_price.len(), 3);
        assert!(quote_gen.quote_to_last_price.contains_key("AAPL"));
        assert!(quote_gen.quote_to_last_price.contains_key("TSLA"));
        assert!(quote_gen.quote_to_last_price.contains_key("GOOGL"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn from_tickers_file_ignores_empty_lines() {
        let path = write_tickers("\n\nAAPL\n\n\n", "empty_lines");
        let quote_gen = QuoteGenerator::from_tickers_file(path.to_str().unwrap()).unwrap();
        assert_eq!(quote_gen.quote_to_last_price.len(), 1);
        assert!(quote_gen.quote_to_last_price.contains_key("AAPL"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn from_tickers_file_missing_returns_err() {
        let result = QuoteGenerator::from_tickers_file("nonexistent_tickers_file_12345.txt");
        assert!(result.is_err());
    }

    #[test]
    fn generate_quote_sets_fields_and_updates_price() {
        let path = write_tickers("AAPL\n", "generate");
        let mut quote_gen = QuoteGenerator::from_tickers_file(path.to_str().unwrap()).unwrap();
        let q1 = quote_gen.generate_quote("AAPL");
        assert_eq!(q1.ticker, "AAPL");
        assert!(q1.price >= 10.0);
        assert!(q1.volume > 0);
        assert!(q1.timestamp > 0);

        let q2 = quote_gen.generate_quote("AAPL");
        assert_eq!(q2.ticker, "AAPL");
        assert!(q2.volume > 0);
        assert!(q2.timestamp > 0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn get_quotes_returns_ticker_list() {
        let path = write_tickers("A\nB\n", "get_quotes");
        let mut generator = QuoteGenerator::from_tickers_file(path.to_str().unwrap()).unwrap();
        let tickers = generator.get_quotes();
        assert_eq!(tickers.len(), 2);
        assert!(tickers.contains(&"A".to_string()));
        assert!(tickers.contains(&"B".to_string()));
        let _ = fs::remove_file(&path);
    }
}
