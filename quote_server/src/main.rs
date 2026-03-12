use crate::generator::QuoteGenerator;

mod generator;
mod error;

fn main() {
    
    let quote_gen = QuoteGenerator::new();
    print!("{:?}", quote_gen);
}
