use crate::error::QuoteServerError;
use crate::generator::QuoteGenerator;
use crate::protocol::StreamRequest;
use quote_lib::StockQuote;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod error;
mod generator;
mod protocol;
type ClientSender = std::sync::mpsc::Sender<StockQuote>;

fn main() -> Result<(), QuoteServerError> {
    env_logger::init();

    let (gen_tx, gen_rx) = std::sync::mpsc::channel::<StockQuote>();
    let quote_gen = QuoteGenerator::new()?;

    let clients: Arc<Mutex<Vec<ClientSender>>> = Arc::new(Mutex::new(Vec::new()));

    let gen_tx_clone = gen_tx.clone();
    let _handle = thread::spawn(move || {
        if let Err(e) = run_generator_loop(quote_gen, gen_tx_clone) {
            log::error!("generator thread finished with error: {e:?}");
        }
    });

    let subscribers = Arc::clone(&clients);

    thread::spawn(move || {
        for quote in gen_rx {
            let mut locked = subscribers.lock().unwrap();
            locked.retain(|sender| sender.send(quote.clone()).is_ok());
        }
    });

    let tcp_listener = TcpListener::bind("127.0.0.1:34254")?;

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    if let Err(e) = handle_tcp_connect(stream, clients_clone) {
                        log::error!("client thread finished with error: {e:?}");
                    }
                });
            },
            Err(e) => {
                log::error!("TCP accept error: {e:?}");
                continue;
            }
        }
    }
    Ok(())
}

fn handle_tcp_connect(tcp_stream: TcpStream, tcp_clients: Arc<Mutex<Vec<ClientSender>>>) -> Result<(), QuoteServerError> {
    let mut stream = tcp_stream;
    let mut buf_reader = BufReader::new(stream.try_clone()?);

    let mut line = String::new();
    let bytes_read = buf_reader.read_line(&mut line)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = match StreamRequest::parse_stream_command(&line) {
        Ok(req) => {
            let _ = stream.write_all(b"OK\n");
            req
        },
        Err(e) => {
            let _ = stream.write_all(b"ERR invalid STREAM command\n");
            return Err(e);
        }
    };

    let (client_tx, client_rx) = mpsc::channel::<StockQuote>();
    {
        let mut locked = tcp_clients.lock().unwrap();
        locked.push(client_tx);
    }

    let udp_addr = request.udp_addr;
    let tickers = request.tickers;

    thread::spawn(move || {
        if let Err(e) = start_client_stream(udp_addr, tickers, client_rx) {
            eprintln!("client stream error: {e:?}");
        }
    });

    Ok(())
}

fn start_client_stream(
    udp_addr: SocketAddr,
    tickers: Vec<String>,
    client_rx: mpsc::Receiver<StockQuote>,
) -> Result<(), QuoteServerError> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let tickers_set: HashSet<String> = tickers.into_iter().collect();

    let last_ping = Arc::new(Mutex::new(Instant::now()));
    let last_ping_for_thread = Arc::clone(&last_ping);

    {
        let socket_ping = socket.try_clone()?;
        thread::spawn(move || {
            let mut buf = [0u8; 64];
            loop {
                match socket_ping.recv_from(&mut buf) {
                    Ok((n, src)) => {
                        if src == udp_addr && &buf[..n] == b"Ping"
                            && let Ok(mut lp) = last_ping_for_thread.lock()
                        {
                            *lp = Instant::now();
                        }
                    }
                    Err(e) => {
                        log::warn!("Ping recv error from {udp_addr}: {e}");
                        break;
                    }
                }
            }
        });
    }

    let timeout = Duration::from_secs(5);

    for quote in client_rx {
        if let Ok(lp) = last_ping.lock() && lp.elapsed() > timeout {
            log::warn!("Ping timeout for {udp_addr}, stopping stream");
            break;
        }

        if !tickers_set.contains(&quote.ticker) {
            continue;
        }

        let bytes = match quote.to_bencode_bytes() {
            Ok(b) => b,
            Err(e) => {
                log::warn!("bencode error for quote {:?}: {e}", quote);
                continue;
            }
        };

        if let Err(e) = socket.send_to(&bytes, udp_addr) {
            log::warn!("UDP send error to {udp_addr}: {e}");
        }
    }

    Ok(())
}

fn run_generator_loop(
    mut generator: QuoteGenerator,
    quotes_txs: mpsc::Sender<StockQuote>,
) -> Result<(), QuoteServerError> {
    loop {
        let tickers = generator.get_quotes();
        for ticker in tickers {
            let quote = generator.generate_quote(&ticker);
            if quotes_txs.send(quote).is_err() {
                return Err(QuoteServerError::DispatcherClosed);
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
