use serde_json;
use openssl::rsa::Rsa;
use pem::{Pem, encode};
use crypto::sha2::Sha256;
use crypto::ripemd160::Ripemd160;
use crypto::digest::Digest;
use rust_base58::{ToBase58, FromBase58};
use rpassword;
use std::io::{Read, Write};
use std::fs::File;
//use crypto::aes::{mod, KeySize};
//use crypto::symmetriccipher::SynchronousStreamCipher;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub addresses:   Vec<String>,
    pub balances:    Vec<i64>,
    pub last_height: usize,
    pub pubkeys:     Vec<Vec<u8>>,
    privkeys:        Vec<String>, // REALLY REALLY REALLY BAD IDEA. MUST REPLACE.
}

impl Wallet {
    pub fn new() -> Result<Wallet, &'static str> {
        
        let mut wallet = Wallet {
            addresses:   vec![],
            balances:    vec![],
            last_height: 0,
            pubkeys:     vec![],
            privkeys:    vec![],
        };

        // I need to store this encrypted in a persistent storage!
        let mut privkeys = vec![];

        println!("Generating public/private keypairs...");
        for i in 0..10 {
            // Generate keypair
            //println!("Generating keypair #{}...", i+1);
            {
                let rsa = Rsa::generate(4096).unwrap();
                wallet.pubkeys.push(rsa.public_key_to_der().unwrap().clone());
                privkeys.push(rsa.private_key_to_der().unwrap().clone());

                let private_pem = Pem {
                    tag: String::from("RSA PRIVKEY"),
                    contents: privkeys[i].clone(),
                };
                let privkey = encode(&private_pem);
                wallet.privkeys.push(privkey.clone()); // TODO: Move to persistent storage
            }
            // Generate binary addresss
            let binaddr = Wallet::generate_binaddr(&wallet.pubkeys[i]);
            wallet.addresses.push(Wallet::generate_address(&binaddr));
            wallet.balances.push(0);
        }

        Ok(wallet)
    }

    pub fn generate_binaddr(pubkey: &Vec<u8>) -> String {
        let sha256digest = {
            let mut hasher = Sha256::new();
            hasher.input(pubkey);
            hasher.result_str()
        };

        let ripemd160digest = {
            let mut hasher = Ripemd160::new();
            hasher.input(&sha256digest.into_bytes());
            hasher.result_str()
        };
        let ripemd160digest = "00".to_owned() + ripemd160digest.as_ref();

        let sha256digest2n3 = {
            let mut hasher1 = Sha256::new();
            let mut hasher2 = Sha256::new();
            hasher1.input(&ripemd160digest.clone().into_bytes());
            hasher2.input(&hasher1.result_str().into_bytes());
            hasher2.result_str()
        };

        let checksum = String::from(&sha256digest2n3[..8]);
        checksum + ripemd160digest.as_ref()
    }

    pub fn generate_address(binaddr: &String) -> String {
        assert_eq!(binaddr.len(), 50);
        
        let mut binvec = vec![];
        for i in 0..25 {
            let hex = &binaddr[(i*2)..(i*2)+2];
            binvec.push(i64::from_str_radix(hex, 16).unwrap() as u8);
        }

        binvec.to_base58()
    }

    pub fn save(&self, filename: &str) -> Result<(), &'static str> {
        // We first serialize our wallet to json.
        match serde_json::to_string(&self) {
            Ok(serialized) => {
                // Input password to encrypt and save wallet privkeys to persistent
                // storage
                let password = rpassword::prompt_password_stdout("Input encryption passphrase: ")
                    .unwrap();
                let sndpass = rpassword::prompt_password_stdout("Please enter passphrase again: ")
                    .unwrap();
                    if password == sndpass {
                        // Encrypt serialized stuff!
                        // Uhn, well, TODO.
                        // Now we open our file
                        let f = File::create(filename);
                        match f {
                            Ok(mut f) => {
                                match f.write_all(serialized.as_bytes()) {
                                    Ok(_) => Ok(()),
                                    _ => Err("Unable to write file.")
                                }
                            },
                            _ => Err("Error opening file.")
                        }
                    }
                else {
                    Err("Passphrases did not match!")
                }
            },
            _ => Err("Error serializing wallet.")
        }
    }

    pub fn load(filename: &str) -> Result<Wallet, &'static str> {
        let f = File::open(filename);
        match f {
            Ok(mut f) => {
                let mut serialized = String::new();
                match f.read_to_string(&mut serialized) {
                    Ok(_) => {
                        match serde_json::from_str(&serialized) {
                            Err(_) => Err("Could not deserialize wallet."),
                            Ok(wallet) => Ok(wallet),
                        }
                    },
                    _ => Err("Could not read file.")
                }
            },
            _ => Err("Could not open file."),
        }
    }
}
