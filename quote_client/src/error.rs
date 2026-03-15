use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteClientError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Tickers file is empty or contains no valid lines")]
    EmptyTickersFile,

    #[error("Failed to connect to server: {0}")]
    TcpConnect(io::Error),

    #[error("Failed to send STREAM command: {0}")]
    TcpWrite(io::Error),

    #[error("Failed to bind UDP socket: {0}")]
    UdpBind(io::Error),
}
