use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    pub fn to_bencode_bytes(&self) -> Result<Vec<u8>, bendy::serde::Error> {
        bendy::serde::to_bytes(self)
    }
    pub fn from_bencode_bytes(bytes: &[u8]) -> Result<Self, bendy::serde::Error> {
        bendy::serde::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::StockQuote;

    #[test]
    fn stock_quote_roundtrip_bencode() {
        let original = StockQuote {
            ticker: "AAPL".to_string(),
            price: 123.45,
            volume: 10_000,
            timestamp: 1_700_000_000_000,
        };

        let encoded = original
            .to_bencode_bytes()
            .expect("encoding to bencode should succeed");

        let decoded =
            StockQuote::from_bencode_bytes(&encoded).expect("decoding from bencode should succeed");

        assert_eq!(decoded.ticker, original.ticker);
        assert!((decoded.price - original.price).abs() < f64::EPSILON);
        assert_eq!(decoded.volume, original.volume);
        assert_eq!(decoded.timestamp, original.timestamp);
    }

    #[test]
    fn stock_quote_roundtrip_bencode_with_zero_values() {
        let original = StockQuote {
            ticker: "".to_string(),
            price: 0.0,
            volume: 0,
            timestamp: 0,
        };

        let encoded = original
            .to_bencode_bytes()
            .expect("encoding to bencode should succeed");

        let decoded =
            StockQuote::from_bencode_bytes(&encoded).expect("decoding from bencode should succeed");

        assert_eq!(decoded.ticker, original.ticker);
        assert!((decoded.price - original.price).abs() < f64::EPSILON);
        assert_eq!(decoded.volume, original.volume);
        assert_eq!(decoded.timestamp, original.timestamp);
    }
}
