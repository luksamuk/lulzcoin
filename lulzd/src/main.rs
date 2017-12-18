#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate crypto;


pub mod chain;

fn main() {
    let mut blockchain = chain::Blockchain::new();
    blockchain.new_transaction(&"me".to_owned(), &"you".to_owned(), 0.000045); // wew lad
    println!("Hello, world!");
    println!("Blockchain serialization:\n{}",
             serde_json::to_string_pretty(&blockchain).unwrap());
}
