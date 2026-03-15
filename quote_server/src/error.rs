use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteServerError {
    #[error("File with tickers not found")]
    Io(#[from] io::Error),

    #[error("generator thread finished with error")]
    DispatcherClosed,

    #[error("Invalid format for stream command, example: STREAM udp://127.0.0.1:34254 AAPL,TSLA")]
    InvalidCommandFormat,

    #[error("Invalid UDP address in STREAM command")]
    InvalidUDPAddress,

    #[error("TCP connection error: {0:?}")]
    TcpConnectionError(std::io::Error),
}
