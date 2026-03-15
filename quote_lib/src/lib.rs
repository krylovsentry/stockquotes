//! # quote_lib
//!
//! Общая библиотека для сервера и клиента котировок: модель котировки и сериализация в [bencode](https://en.wikipedia.org/wiki/Bencode) (через [bendy](https://docs.rs/bendy)).

use serde::{Deserialize, Serialize};

/// Котировка по одному тикеру: тикер, цена, объём и метка времени.
///
/// Используется для передачи по UDP между сервером и клиентом в формате bencode.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockQuote {
    /// Символьный код инструмента (например, `"AAPL"`, `"TSLA"`).
    pub ticker: String,
    /// Цена.
    pub price: f64,
    /// Объём (количество в сделке/периоде).
    pub volume: u32,
    /// Временная метка (Unix timestamp, секунды или миллисекунды в зависимости от источника).
    pub timestamp: u64,
}

impl StockQuote {
    /// Сериализует котировку в байты в формате bencode.
    ///
    /// Результат можно отправлять по UDP или сохранять. Ошибка возвращается при сбое сериализации.
    pub fn to_bencode_bytes(&self) -> Result<Vec<u8>, bendy::serde::Error> {
        bendy::serde::to_bytes(self)
    }

    /// Десериализует котировку из байтов в формате bencode.
    ///
    /// Обычно вызывается на стороне клиента после `recv_from`. Ошибка возвращается при невалидных или неполных данных.
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
