extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

const INTERNETZ_VERSION: &'static str = env!("CARGO_PKG_VERSION");
static COMMANDS: &'static str =
    "quit  -- Quits application.\n\
     help  -- Shows this help prompt.\n";


fn main() {
    println!("internetz {}", INTERNETZ_VERSION);
    println!("CLI wallet for lulzcoin, the unimaginably useless cryptocoin.");
    println!("Copyright (C) 2017 Lucas Vieira.");
    println!("This program is distributed under the MIT License. Check source code for details.");
    
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
