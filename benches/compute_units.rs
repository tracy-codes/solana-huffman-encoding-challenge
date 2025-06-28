use mollusk_svm::Mollusk;
use mollusk_svm_bencher::MolluskComputeUnitBencher;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;

// Import the encoder from the test module structure
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuffmanTree {
    Leaf {
        freq: u32,
        byte: u8,
    },
    Node {
        freq: u32,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
    },
}

impl HuffmanTree {
    fn freq(&self) -> u32 {
        match self {
            HuffmanTree::Leaf { freq, .. } => *freq,
            HuffmanTree::Node { freq, .. } => *freq,
        }
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.freq().cmp(&self.freq()) // Reverse for min-heap
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HuffmanEncoder {
    codes: HashMap<u8, (u32, u8)>, // byte -> (code, bit_length)
    tree_bytes: Vec<u8>,           // Serialized tree for decoder
}

impl HuffmanEncoder {
    pub fn new(input: &str) -> Self {
        let frequencies = Self::calculate_frequencies(input);
        let tree = Self::build_tree(&frequencies);
        let mut codes = HashMap::new();

        if frequencies.len() == 1 {
            // Special case: single character
            let byte = frequencies.keys().next().unwrap();
            codes.insert(*byte, (0, 1));
        } else {
            Self::generate_codes(&tree, &mut codes, 0, 0);
        }

        let tree_bytes = Self::serialize_tree(&tree);

        Self { codes, tree_bytes }
    }

    fn calculate_frequencies(input: &str) -> HashMap<u8, u32> {
        let mut frequencies = HashMap::new();
        for byte in input.bytes() {
            *frequencies.entry(byte).or_insert(0) += 1;
        }
        frequencies
    }

    fn build_tree(frequencies: &HashMap<u8, u32>) -> HuffmanTree {
        let mut heap = BinaryHeap::new();

        // Create leaf nodes
        for (&byte, &freq) in frequencies {
            heap.push(HuffmanTree::Leaf { freq, byte });
        }

        // Build tree
        while heap.len() > 1 {
            let right = heap.pop().unwrap();
            let left = heap.pop().unwrap();
            let freq = left.freq() + right.freq();
            heap.push(HuffmanTree::Node {
                freq,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        heap.pop().unwrap()
    }

    fn generate_codes(
        tree: &HuffmanTree,
        codes: &mut HashMap<u8, (u32, u8)>,
        code: u32,
        depth: u8,
    ) {
        match tree {
            HuffmanTree::Leaf { byte, .. } => {
                codes.insert(*byte, (code, depth.max(1)));
            }
            HuffmanTree::Node { left, right, .. } => {
                Self::generate_codes(left, codes, code << 1, depth + 1);
                Self::generate_codes(right, codes, (code << 1) | 1, depth + 1);
            }
        }
    }

    fn serialize_tree(tree: &HuffmanTree) -> Vec<u8> {
        let mut bytes = Vec::new();
        Self::serialize_tree_recursive(tree, &mut bytes);
        bytes
    }

    fn serialize_tree_recursive(tree: &HuffmanTree, bytes: &mut Vec<u8>) {
        match tree {
            HuffmanTree::Leaf { byte, .. } => {
                bytes.push(1); // Leaf marker
                bytes.push(*byte);
            }
            HuffmanTree::Node { left, right, .. } => {
                bytes.push(0); // Internal node marker
                Self::serialize_tree_recursive(left, bytes);
                Self::serialize_tree_recursive(right, bytes);
            }
        }
    }

    pub fn encode(&self, input: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_count = 0u8;

        // First, write the tree
        result.extend_from_slice(&(self.tree_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.tree_bytes);

        // Then encode the data
        for byte in input.bytes() {
            if let Some(&(code, bit_length)) = self.codes.get(&byte) {
                for i in (0..bit_length).rev() {
                    let bit = ((code >> i) & 1) as u8;
                    current_byte |= bit << (7 - bit_count);
                    bit_count += 1;

                    if bit_count == 8 {
                        result.push(current_byte);
                        current_byte = 0;
                        bit_count = 0;
                    }
                }
            }
        }

        // Push remaining bits
        if bit_count > 0 {
            result.push(current_byte);
        }

        result
    }
}

pub fn huffman_encode_url(url: &str) -> Vec<u8> {
    let encoder = HuffmanEncoder::new(url);
    encoder.encode(url)
}

// Benchmark utilities
pub const PROGRAM: Pubkey = Pubkey::new_from_array(solana_huffman_encoding_challenge::ID);

pub fn mollusk() -> Mollusk {
    Mollusk::new(&PROGRAM, "target/deploy/solana_huffman_encoding_challenge")
}

pub fn create_instruction_data(encoded_url: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&(encoded_url.len() as u32).to_le_bytes());
    data.extend_from_slice(encoded_url);
    data
}

fn benchmark_url(mollusk: &Mollusk, url: &str) -> (Instruction, Vec<(Pubkey, Account)>) {
    let encoded_url = huffman_encode_url(url);
    let instruction_data = create_instruction_data(&encoded_url);

    let ix_accounts = vec![];
    let ix = Instruction::new_with_bytes(PROGRAM, &instruction_data, ix_accounts);
    let tx_accounts = vec![];
    (ix, tx_accounts)
}

fn main() {
    let mollusk = mollusk();

    // Challenge URLs from README
    let test_urls = vec![
        "http://localhost:3000",
        "http://subdomain.localhost:3000",
        "https://localhost.net",
        "https://google.com",
        "https://a.a",
        "https://a.com",
        "https://git@github.com:username/repo.git",
        "https://a-really-long-url-that-probably-would-be-so-hard-to-actually-use-but-whatever.com",
        "https://🦝👀🍹🌏.net",
        "https://something.yourcooldomain.com?query_param=123&val=true",
    ];

    let mut results = vec![];

    for url in test_urls {
        let (ix, tx_accounts) = benchmark_url(&mollusk, &url);

        results.push((url, ix, tx_accounts));
    }

    let mut bencher = MolluskComputeUnitBencher::new(mollusk);
    for (url, ix, tx_accounts) in &results {
        bencher = bencher.bench((url, ix, tx_accounts));
    }
    bencher.must_pass(true).out_dir("benches/").execute();
}
