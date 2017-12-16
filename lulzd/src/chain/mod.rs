use time;
use serde_json;
use std::collections::HashSet;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub struct Node {
    identifier: String,
}


// ---

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    sender: String,
    recipient: String,
    amount: u64,
}

// ---

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHdr {
    index: u64,
    timestamp: u64,
    pow: u64,
    prevhash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    header: BlockHdr,
    transactions: Vec<Transaction>,
}

// ---

#[derive(Serialize, Deserialize, Clone)]
pub struct Blockchain {
    chain: Vec<Block>,
    known_nodes: HashSet<String>,
    pending: Vec<Transaction>,
}


impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain:       vec![],
            known_nodes: HashSet::new(),
            pending:     vec![],
        };
        // Create genesis block
        blockchain.new_block(100, None);
        blockchain
    }

    pub fn new_block(&mut self, pow: u64, prevhash: Option<String>) -> &Block {
        let block = Block {
            header: BlockHdr {
                index:     self.chain.len() as u64 + 1,
                timestamp: time::precise_time_ns(),
                pow:       pow,
                prevhash:  match prevhash {
                    Some(hash) => hash,
                    None       => "1".to_owned(),
                },
            },
            transactions: self.pending.clone(),
            
        };

        self.pending.clear();
        self.chain.push(block.clone());
        self.chain.last().unwrap()
    }

    pub fn hash(block: &Block) -> String {
        let serialized = serde_json::to_string(&block).unwrap();
        let bytes      = serialized.into_bytes();
        
        let mut hasher = Sha256::new();
        hasher.input(&bytes);
        hasher.result_str()
    }
}
