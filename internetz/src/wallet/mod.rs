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
use std::fs::metadata;
use rand::{thread_rng, Rng};
use crypto::aes::{ctr, KeySize};
use crypto::symmetriccipher::SynchronousStreamCipher;
//use crypto::scrypt::*;
use crypto::bcrypt::bcrypt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub addresses:   Vec<String>,
    pub balances:    Vec<f64>,
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
            wallet.balances.push(0.0);
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
                        // Generate a nice key here.
                        let mut key = [0u8; 24];
                        {
                            // Our salt is "jt)oVdr42&8*r~?", with one byte per
                            // letter, but in Rust, we declare the bytevec already.
                            // Passing this string through xxd yields:
                            // 00000000: 6a74 296f 5664 7234 3226 382a 727e 3f26  jt)oVdr42&8*r~?&
                            // 00000010: 0a                                       .
                            //
                            // The extra . is the end of string, which we discard.
                            let salt = [0x6a, 0x74, 0x29, 0x6f, 0x56, 0x64, 0x72, 0x34,
                                        0x32, 0x26, 0x38, 0x2a, 0x72, 0x7e, 0x3f, 0x26];
                            /*scrypt(password.as_bytes(),
                                   salt,
                                   &ScryptParams::new(255, 8, 1),
                            &mut key);*/
                            bcrypt(11, &salt, password.as_bytes(), &mut key);
                                   
                        }

                        // Generate random data to be our nonce.
                        let mut nonce = [0u8; 24]; //Vec::with_capacity(256);
                        thread_rng().fill_bytes(&mut nonce);
                        
                        // Since "key" can be considered a well-formed, key, all we need to do is
                        // encrypt our bytes using it, then write the generated buffer to our file.
                        println!("Encrypting...");
                        let mut cipher = ctr(KeySize::KeySize192, &key, &nonce);
                        let mut encrypted: Vec<u8> = {
                            let mut vec = Vec::with_capacity(serialized.as_bytes().len() as usize);
                            for _ in 0..serialized.as_bytes().len() {
                                vec.push(0);
                            }
                            vec
                        };
                        cipher.process(&serialized.as_bytes(), encrypted.as_mut_slice());

                        // Open a file to save the nonce.
                        let n = File::create(String::from(filename) + ".nonce");
                        match n {
                            Ok(mut n) => {
                                match n.write_all(&nonce) {
                                    Ok(_) => {
                                        // Now we open our file to save the wallet.
                                        let f = File::create(filename);
                                        match f {
                                            Ok(mut f) => {
                                                match f.write_all(encrypted.as_slice()) { // serialized.as_bytes()
                                                    Ok(_) => Ok(()),
                                                    _ => Err("Unable to write wallet file.")
                                                }
                                            },
                                            _ => Err("Error opening file.")
                                        }
                                    },
                                    _ => Err("Unable to write unique number.")
                                }
                            },
                            _ => Err("Error opening file."),
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
        let mut nonce = [0u8; 24];
        
        let n = File::open(String::from(filename) + ".nonce");
        let f = File::open(filename);

        // First check for our nonce.
        match n {
            Ok(mut n) => {
                match n.read(&mut nonce) {
                    Ok(_) => {
                        // Now read our file
                        match f {
                            Ok(mut f) => {
                                let mut serialized = String::new();
                                let metadata = metadata(filename).unwrap();
                                let mut encrypted: Vec<u8> = {
                                    let mut vec = Vec::with_capacity(metadata.len() as usize);
                                    for _ in 0..metadata.len() {
                                        vec.push(0);
                                    }
                                    vec
                                };
                                
                                match f.read(&mut encrypted) {
                                    Ok(_) => {
                                        // First, decrypt!
                                        println!("Loaded file size: {} bytes", encrypted.len());
                                        panic!("NOT IMPLEMENTED! STOP TRYING TO LOAD!");
                                        // Now, just deserialize successfully.
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
                        
                    },
                    _ => Err("Could not read unique number."),
                }
            },
            _ => Err("Could not open file."),
        }
    }
}
