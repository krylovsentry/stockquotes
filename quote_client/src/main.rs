mod error;

use crate::error::QuoteClientError;
use clap::Parser;
use quote_lib::StockQuote;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Клиент котировок: подключается к серверу и получает поток котировок по UDP.
#[derive(Parser, Debug)]
#[command(name = "quote_client")]
#[command(author, version, about)]
pub struct Config {
    /// Адрес и порт TCP-сервера (например, 127.0.0.1:34254)
    #[arg(long, value_name = "ADDR")]
    pub server_addr: String,

    /// Порт для приёма UDP-котировок на стороне клиента
    #[arg(long, value_name = "PORT")]
    pub udp_port: u16,

    /// Путь к файлу со списком тикеров (по одному на строку)
    #[arg(long, value_name = "FILE")]
    pub tickers_file: String,
}

/// Загружает список тикеров из файла (по одному на строку, пустые строки пропускаются).
pub(crate) fn load_tickers(path: &str) -> Result<Vec<String>, QuoteClientError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tickers: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(tickers)
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        log::error!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), QuoteClientError> {
    let config = Config::parse();

    let tickers = load_tickers(&config.tickers_file)?;
    if tickers.is_empty() {
        return Err(QuoteClientError::EmptyTickersFile);
    }

    let mut stream = TcpStream::connect(&config.server_addr).map_err(QuoteClientError::TcpConnect)?;

    // Адрес, на который сервер будет слать UDP (для локального теста — 127.0.0.1)
    let udp_uri = format!("udp://127.0.0.1:{}", config.udp_port);
    let tickers_csv = tickers.join(",");
    let command = format!("STREAM {} {}\n", udp_uri, tickers_csv);

    stream
        .write_all(command.as_bytes())
        .map_err(QuoteClientError::TcpWrite)?;
    stream.flush().map_err(QuoteClientError::TcpWrite)?;

    log::info!(
        "Sent STREAM command for {} ticker(s) to {}",
        tickers.len(),
        config.server_addr
    );

    let socket = UdpSocket::bind(("0.0.0.0", config.udp_port)).map_err(QuoteClientError::UdpBind)?;
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(QuoteClientError::UdpBind)?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::SeqCst);
    })
    .expect("failed to set Ctrl+C handler");

    let mut server_udp_addr = None;
    let mut buf = [0u8; 2048];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if server_udp_addr.is_none() {
                    server_udp_addr = Some(from);
                    let ping_addr = from;
                    let shutdown_ping = Arc::clone(&shutdown);
                    let socket_ping = socket.try_clone().map_err(QuoteClientError::UdpBind)?;
                    thread::spawn(move || {
                        while !shutdown_ping.load(Ordering::SeqCst) {
                            let _ = socket_ping.send_to(b"Ping", ping_addr);
                            for _ in 0..20 {
                                if shutdown_ping.load(Ordering::SeqCst) {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                    });
                }
                if let Ok(quote) = StockQuote::from_bencode_bytes(&buf[..n]) {
                    println!(
                        "{} price={} volume={} ts={}",
                        quote.ticker, quote.price, quote.volume, quote.timestamp
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(QuoteClientError::Io(e)),
        }
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
    }

    log::info!("Shutting down.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_tickers;
    use crate::error::QuoteClientError;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn write_tickers_to_path(content: &str, path: &PathBuf) {
        let mut f = File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.sync_all().unwrap();
    }

    #[test]
    fn load_tickers_reads_lines() {
        let p = std::env::temp_dir().join("quote_client_test_reads_lines.txt");
        write_tickers_to_path("AAPL\nTSLA\nGOOGL\n", &p);
        let tickers = load_tickers(p.to_str().unwrap()).unwrap();
        assert_eq!(tickers, &["AAPL", "TSLA", "GOOGL"]);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn load_tickers_ignores_empty_lines() {
        let p = std::env::temp_dir().join("quote_client_test_ignores_empty.txt");
        write_tickers_to_path("\n\nAAPL\n\n\n", &p);
        let tickers = load_tickers(p.to_str().unwrap()).unwrap();
        assert_eq!(tickers, &["AAPL"]);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn load_tickers_trims_whitespace() {
        let p = std::env::temp_dir().join("quote_client_test_trims.txt");
        write_tickers_to_path("  AAPL  \n  TSLA  \n", &p);
        let tickers = load_tickers(p.to_str().unwrap()).unwrap();
        assert_eq!(tickers, &["AAPL", "TSLA"]);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn load_tickers_missing_file_returns_err() {
        let result = load_tickers("nonexistent_tickers_12345.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QuoteClientError::Io(_)));
    }

    #[test]
    fn load_tickers_empty_file_returns_empty_vec() {
        let p = std::env::temp_dir().join("quote_client_test_empty.txt");
        write_tickers_to_path("", &p);
        let tickers = load_tickers(p.to_str().unwrap()).unwrap();
        assert!(tickers.is_empty());
        let _ = std::fs::remove_file(&p);
    }
}
