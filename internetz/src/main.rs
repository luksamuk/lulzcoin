extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

const INTERNETZ_VERSION: &'static str = env!("CARGO_PKG_VERSION");
static COMMANDS: &'static str =
    "sair  -- Sai da aplicação.\n\
     ajuda -- Mostra esta tabela.\n";


fn main() {
    println!("internetz {}", INTERNETZ_VERSION);
    println!("Carteira CLI para lulzcoin, a moeda inimaginavelmente inútil.");
    println!("Copyright (C) 2017 Lucas Vieira.");
    println!("This program is distributed under the MIT License. Check source code for details.");
    
    // Editor
    let mut rl = Editor::<()>::new();

    // Load REPL history
    if let Err(_) = rl.load_history(".internetz-history") {
        println!("Nenhum comando no histórico.");
    }

    println!("Para uma lista de comandos, digite `ajuda`.");
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
                        "sair"  => break,
                        "ajuda" => println!("LISTA DE COMANDOS:\n{}", COMMANDS),
                        _       => println!("Nao implementado"),
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("Recebido um sinal de interrupção. Saindo.");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("Recebido um sinal de fim de arquivo. Saindo.");
                break;
            },
            Err(err) => {
                println!("Erro de leitura de linha: {:?}. Saindo.", err);
                break;
            },
        }
    }

    rl.save_history(".internetz-history").unwrap();
    
}
