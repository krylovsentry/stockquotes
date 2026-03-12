use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteServerError {
    #[error("File with tickers not found")]
    Io(#[from] io::Error),
}
