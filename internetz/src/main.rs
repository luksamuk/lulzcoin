#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate crypto;
extern crate openssl;
extern crate pem;
extern crate rust_base58;
extern crate rpassword;

extern crate rustyline;



pub mod wallet;
use wallet::Wallet;



use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

const INTERNETZ_VERSION: &'static str = env!("CARGO_PKG_VERSION");
static COMMANDS: &'static str =
    "quit   -- Quits application.\n\
     create -- [WIP] Generates a new wallet.\n\
     open   -- [PLANNED] Opens a wallet.\n\
     help   -- Shows this help prompt.\n";


fn main() {
    println!("internetz {}", INTERNETZ_VERSION);
    println!("CLI wallet for lulzcoin, the unimaginably useless cryptocoin.");
    println!("Copyright (C) 2017 Lucas Vieira.");
    println!("This program is distributed under the MIT License. Check source code for details.");

    // Ensure lulzcoin dir exists
    let _ = fs::create_dir_all(".lulzcoin");

    // Load wallet data
    println!("THIS IS A BETA! Expect bugs here and there on this wallet. Plus, please, do not");
    println!("consider this a safe software; even though there are some safety measures implemented,");
    println!("this software still cannot hold your money safely.");
    
    // Editor
    let mut rl = Editor::<()>::new();

    // Load REPL history
    if let Err(_) = rl.load_history(".internetz-history") {
        println!("No command on history.");
    }

    println!("For a list of commands, type `help`.");
    loop {
        let readline = rl.readline("INTERNETZ > ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let atoms = line.split_whitespace()
                    .collect::<Vec<&str>>();

                if atoms.len() > 0 {
                    // Habemus input.
                    let command = String::from(atoms[0]).to_lowercase();
                    let args = &atoms[1..];

                    match command.as_ref() {
                        "quit" | "exit"  => break,
                        "help" => {
                            println!("You're gonna get paid in INTERNETZ: Worth more than a bar of lulz.");
                            println!("COMMAND LIST:\n{}", COMMANDS);
                        },
                        "create" => {
                            // Retrieve name for wallet
                            let walletname = rl.readline("Please input a wallet name: ").unwrap();
                            let filename = String::from("./.lulzcoin/") + walletname.as_ref();
                            let filename = filename + ".lulz";
                            
                            println!("Your wallet will be stored in ./.lulzcoin/{}.lulz.",
                                     walletname);

                            // Wallet test
                            match Wallet::new() {
                                Ok(wallet) => {
                                    println!("Wallet successfully generated. Your addresses:");
                                    for addr in &wallet.addresses {
                                        println!("\t{}", addr);
                                    }
                                    match wallet.save(filename.as_ref()) {
                                        Ok(_) => println!("Wallet saved in {}.", filename),
                                        Err(error) => println!("Error saving wallet: {}", error),
                                    }
                                    
                                },
                                Err(error) => {
                                    println!("Error generating wallet: {}", error);
                                },
                            }
                            
                        },
                        _ => println!("Nao implementado"),
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("Interrupt received. Quitting.");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("EOF received. Quitting.");
                break;
            },
            Err(err) => {
                println!("Error reading line: {:?}. Quitting.", err);
                break;
            },
        }
    }

    rl.save_history(".internetz-history").unwrap();
    
}
