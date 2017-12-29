use serde_json;
use openssl::rsa::Rsa;
use crypto::sha2::Sha256;
use crypto::ripemd160::Ripemd160;
use crypto::digest::Digest;
use rust_base58::{ToBase58, FromBase58};
use rpassword;
use std::io::{Read, Write};
use std::fs::File;
use std::fs::metadata;
use hex::{FromHex, ToHex};
use std::str;

use sodiumoxide::crypto::aead;
use sodiumoxide::crypto::pwhash::scryptsalsa208sha256::{Salt, OpsLimit, MemLimit, derive_key};


// Our salt is "jt)oVdr42&8*r~?&", with one byte per
// letter, but in Rust, we declare the bytevec already.
// Passing this string through xxd yields:
// 00000000: 6a74 296f 5664 7234 3226 382a 727e 3f26  jt)oVdr42&8*r~?&
// 00000010: 0a                                       .
//
// The extra . is the end of string, which we discard.
// NOTE: We doubled the salt for debug purposes. We need to replace this with
// a proper 32-bit salt.
//const SALT: [u8; 16] = [0x6a, 0x74, 0x29, 0x6f, 0x56, 0x64, 0x72, 0x34,
//                        0x32, 0x26, 0x38, 0x2a, 0x72, 0x7e, 0x3f, 0x26];
const SALT: [u8; 32] = [0x6a, 0x74, 0x29, 0x6f, 0x56, 0x64, 0x72, 0x34,
                        0x32, 0x26, 0x38, 0x2a, 0x72, 0x7e, 0x3f, 0x26,
                        0x6a, 0x74, 0x29, 0x6f, 0x56, 0x64, 0x72, 0x34,
                        0x32, 0x26, 0x38, 0x2a, 0x72, 0x7e, 0x3f, 0x26];




#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub addresses:   Vec<String>,
    pub balances:    Vec<f64>,
    pub last_height: usize,
    pub pubkeys:     Vec<String>,
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

        println!("Generating public/private keypairs...");
        for _ in 0..10 {
            // Generate keypair
            //println!("Generating keypair #{}...", i+1);
            {
                let rsa = Rsa::generate(4096).unwrap();
                let pubkey = rsa.public_key_to_der().unwrap();
                let privkey = rsa.private_key_to_der().unwrap();

                // Generate new address
                let binaddr = Wallet::generate_binaddr(&pubkey);
                wallet.addresses.push(Wallet::generate_address(&binaddr));
                wallet.balances.push(0.0);
                
                // Keys are converted to hex strings.
                // You can convert them back to Vec by using String::from_hex, impl
                // by the hex crate.
                wallet.pubkeys.push(Vec::to_hex(&pubkey));
                wallet.privkeys.push(Vec::to_hex(&privkey));
            }
            // Generate binary addresss
            
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
                        // using sodiumoxide
                        let mut key = [0u8; 32];
                        let _ = derive_key(&mut key, password.as_bytes(), &Salt(SALT), OpsLimit(11), MemLimit(16*1024));
                        let key = aead::Key(key);

                        let nonce = aead::gen_nonce();
                        
                        // Since "key" can be considered a well-formed, key, all we need to do is
                        // encrypt our bytes using it, then write the generated buffer to our file.
                        println!("Encrypting...");

                        // using sodiumoxide
                        let encrypted = aead::seal(serialized.as_bytes(), None, &nonce, &key);
                        println!("Some bytes: {:?}", &encrypted[..8]);

                        // Open a file to save the nonce.
                        let n = File::create(String::from(filename) + ".nonce");
                        match n {
                            Ok(mut n) => {
                                match n.write_all(nonce.as_ref()) {
                                    Ok(_) => {
                                        // Now we open our file to save the wallet.
                                        let f = File::create(filename);
                                        match f {
                                            Ok(mut f) => {
                                                match f.write_all(encrypted.as_slice()) {
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
        let n = File::open(String::from(filename) + ".nonce");
        let f = File::open(filename);

        // First check for our nonce.
        match n {
            Ok(mut n) => {
                let mut nonce = [0u8; 12];
                match n.read(&mut nonce) {
                    Ok(_) => {
                        match f {
                            Ok(mut f) => {
                                //let mut serialized = String::new();
                                let metadata = metadata(filename).unwrap();
                                let mut encrypted = {
                                    let mut vec = Vec::with_capacity(metadata.len() as usize);
                                    for _ in 0..metadata.len() {
                                        vec.push(0);
                                    }
                                    vec
                                };
                                
                                match f.read(&mut encrypted) {
                                    Ok(_) => {
                                        // Generate key
                                        let mut key = [0u8; 32];
                                        {
                                            let password = rpassword::prompt_password_stdout("Input decryption passphrase: ")
                                                .unwrap();
                                            let _ = derive_key(&mut key, password.as_bytes(), &Salt(SALT), OpsLimit(11), MemLimit(16*1024));
                                        }
                                        let key = aead::Key(key);

                                        // Decrypt
                                        match aead::open(&encrypted, None, &aead::Nonce(nonce), &key){
                                            Ok(decrypted) => {
                                                let serialized = String::from_utf8_lossy(&decrypted);

                                                match serde_json::from_str(&serialized) {
                                                    Err(_) => Err("Could not deserialize wallet."),
                                                    Ok(wallet) => Ok(wallet),
                                                }
                                            },
                                            _ => Err("Could not decrypt wallet file."),
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
