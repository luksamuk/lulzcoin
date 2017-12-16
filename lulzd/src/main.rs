#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate crypto;


pub mod chain;

fn main() {
    let mut blockchain = chain::Blockchain::new();
    println!("Hello, world!");
    println!("Blockchain serialization:\n{}",
             serde_json::to_string_pretty(&blockchain).unwrap());
}
