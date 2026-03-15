use crate::error::QuoteServerError;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct StreamRequest {
    pub udp_addr: SocketAddr,
    pub tickers: Vec<String>,
}

const PREFIX: &str = "STREAM ";

impl StreamRequest {
    pub fn parse_stream_command(line: &str) -> Result<StreamRequest, QuoteServerError> {
        let trimmed = line.trim();
        if !trimmed.starts_with(PREFIX) {
            return Err(QuoteServerError::InvalidCommandFormat);
        }

        let rest_trimmed = &trimmed[PREFIX.len()..];
        // "udp://127.0.0.1:34254 AAPL,TSLA"
        let (uri_part, tickers_part) = rest_trimmed
            .split_once(' ')
            .ok_or(QuoteServerError::InvalidCommandFormat)?;

        const UDP_PREFIX: &str = "udp://";

        if !uri_part.starts_with(UDP_PREFIX) {
            return Err(QuoteServerError::InvalidUDPAddress);
        }

        let address_for_udp = &uri_part[UDP_PREFIX.len()..];

        let udp_addr: SocketAddr = address_for_udp
            .parse()
            .map_err(|_| QuoteServerError::InvalidUDPAddress)?;

        let tickers: Vec<String> = tickers_part
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        if tickers.is_empty() {
            return Err(QuoteServerError::InvalidCommandFormat);
        }

        Ok(StreamRequest { udp_addr, tickers })
    }
}

#[cfg(test)]
mod tests {
    use super::StreamRequest;
    use crate::error::QuoteServerError;
    use std::net::SocketAddr;

    #[test]
    fn parse_stream_command_valid() {
        let req =
            StreamRequest::parse_stream_command("STREAM udp://127.0.0.1:34254 AAPL,TSLA\n").unwrap();
        assert_eq!(req.udp_addr, "127.0.0.1:34254".parse::<SocketAddr>().unwrap());
        assert_eq!(req.tickers, &["AAPL", "TSLA"]);
    }

    #[test]
    fn parse_stream_command_single_ticker() {
        let req =
            StreamRequest::parse_stream_command("STREAM udp://0.0.0.0:40000 GOOGL").unwrap();
        assert_eq!(req.udp_addr, "0.0.0.0:40000".parse::<SocketAddr>().unwrap());
        assert_eq!(req.tickers, &["GOOGL"]);
    }

    #[test]
    fn parse_stream_command_tickers_with_spaces() {
        let req = StreamRequest::parse_stream_command("STREAM udp://127.0.0.1:9999  AAPL , TSLA ")
            .unwrap();
        assert_eq!(req.tickers, &["AAPL", "TSLA"]);
    }

    #[test]
    fn parse_stream_command_missing_prefix() {
        let err = StreamRequest::parse_stream_command("udp://127.0.0.1:34254 AAPL").unwrap_err();
        assert!(matches!(err, QuoteServerError::InvalidCommandFormat));
    }

    #[test]
    fn parse_stream_command_invalid_udp_uri() {
        let err =
            StreamRequest::parse_stream_command("STREAM tcp://127.0.0.1:34254 AAPL").unwrap_err();
        assert!(matches!(err, QuoteServerError::InvalidUDPAddress));
    }

    #[test]
    fn parse_stream_command_invalid_address() {
        let err = StreamRequest::parse_stream_command("STREAM udp://invalid:port AAPL").unwrap_err();
        assert!(matches!(err, QuoteServerError::InvalidUDPAddress));
    }

    #[test]
    fn parse_stream_command_empty_tickers() {
        let err =
            StreamRequest::parse_stream_command("STREAM udp://127.0.0.1:34254 ").unwrap_err();
        assert!(matches!(err, QuoteServerError::InvalidCommandFormat));
    }

    #[test]
    fn parse_stream_command_only_commas_tickers() {
        let err =
            StreamRequest::parse_stream_command("STREAM udp://127.0.0.1:34254 , , ").unwrap_err();
        assert!(matches!(err, QuoteServerError::InvalidCommandFormat));
    }
}
