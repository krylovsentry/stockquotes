use crate::error::QuoteServerError;
use crate::generator::QuoteGenerator;
use quote_lib::StockQuote;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

mod error;
mod generator;
mod protocol;

fn main() {
    let (gen_tx, gen_rx) = std::sync::mpsc::channel::<StockQuote>();
    let mut quote_gen = QuoteGenerator::new();
    type ClientSender = std::sync::mpsc::Sender<StockQuote>;

    let clients: Arc<Mutex<Vec<ClientSender>>> = Arc::new(Mutex::new(Vec::new()));

    let gen_tx_clone = gen_tx.clone();
    let handle = thread::spawn(move || {
        if let Err(e) = run_generator_loop(quote_gen, gen_tx_clone) {
            eprintln!("generator thread finished with error: {e:?}");
        }
    });

    let subscribers = Arc::clone(&clients);

    thread::spawn(move || {
        for quote in gen_rx {
            let mut locked = subscribers.lock().unwrap();
            locked.retain(|sender| sender.send(quote.clone()).is_ok());
        }
    });


    let (client_tx, client_rx) = mpsc::channel::<StockQuote>();
    {
        let mut locked = clients.lock().unwrap();
        locked.push(client_tx);
    }
    // читаем несколько котировок и печатаем
    for _ in 0..10 {
        if let Ok(q) = client_rx.recv() {
            println!("TEST CLIENT GOT QUOTE: {:?}", q);
        }
    }
}

fn run_generator_loop(mut generator: QuoteGenerator, quotes_txs: mpsc::Sender<StockQuote>) -> Result<(), QuoteServerError> {
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