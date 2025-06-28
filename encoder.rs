// This is the encoder program

use std::collections::{BTreeMap, BinaryHeap, HashMap};

// UTF-8-safe prefix/suffix markers
const PREFIX_MARKER: char = '\u{00F1}'; // ñ
const SUFFIX_MARKER: char = '\u{00F2}'; // ò

const PREFIX_LIST: &[(&str, char)] = &[
    ("https://www.", '1'),
    ("https://", '2'),
    ("https://localhost", '3'),
    ("http://", '4'),
    ("http://localhost", '5'),
];

const SUFFIX_LIST: &[(&str, char)] = &[
    (".com", '1'),
    (".org", '2'),
    (".net", '3'),
    ("/index.html", '4'),
    (".git", '5'),
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum HuffNode {
    Leaf {
        frequency: u32,
        symbol: char,
    },
    Branch {
        frequency: u32,
        left: Box<HuffNode>,
        right: Box<HuffNode>,
    },
}

impl HuffNode {
    fn weight(&self) -> u32 {
        match self {
            HuffNode::Leaf { frequency, .. } => *frequency,
            HuffNode::Branch { frequency, .. } => *frequency,
        }
    }
}

impl Ord for HuffNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.weight().cmp(&self.weight())
    }
}

impl PartialOrd for HuffNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct UrlCompressor {
    code_map: HashMap<char, (u32, u8)>,
    serialized_tree: String,
}

impl UrlCompressor {
    fn from_input(input: &str) -> Self {
        let mut freq = BTreeMap::new();
        for ch in input.chars() {
            *freq.entry(ch).or_insert(0) += 1;
        }

        let tree = Self::build_tree(&freq);
        let mut code_map = HashMap::new();

        if freq.len() == 1 {
            let only = *freq.keys().next().unwrap();
            code_map.insert(only, (0, 1));
        } else {
            Self::assign_codes(&tree, &mut code_map, 0, 0);
        }

        let serialized = Self::serialize(&tree);
        Self {
            code_map,
            serialized_tree: serialized,
        }
    }

    fn build_tree(freq: &BTreeMap<char, u32>) -> HuffNode {
        let mut heap = BinaryHeap::new();
        for (&ch, &f) in freq {
            heap.push(HuffNode::Leaf {
                frequency: f,
                symbol: ch,
            });
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(HuffNode::Branch {
                frequency: left.weight() + right.weight(),
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        heap.pop().unwrap()
    }

    fn assign_codes(node: &HuffNode, map: &mut HashMap<char, (u32, u8)>, path: u32, depth: u8) {
        match node {
            HuffNode::Leaf { symbol, .. } => {
                map.insert(*symbol, (path, depth.max(1)));
            }
            HuffNode::Branch { left, right, .. } => {
                Self::assign_codes(left, map, path << 1, depth + 1);
                Self::assign_codes(right, map, (path << 1) | 1, depth + 1);
            }
        }
    }

    fn serialize(node: &HuffNode) -> String {
        let mut out = String::new();
        Self::walk_serialize(node, &mut out);
        out
    }

    fn walk_serialize(node: &HuffNode, out: &mut String) {
        match node {
            HuffNode::Leaf { symbol, .. } => {
                out.push('1');
                out.push(*symbol);
            }
            HuffNode::Branch { left, right, .. } => {
                out.push('0');
                Self::walk_serialize(left, out);
                Self::walk_serialize(right, out);
            }
        }
    }

    fn encode_bytes(&self, input: &str) -> Vec<u8> {
        let mut bits = Vec::new();

        for ch in input.chars() {
            if let Some(&(code, len)) = self.code_map.get(&ch) {
                for i in (0..len).rev() {
                    bits.push(((code >> i) & 1) as u8);
                }
            }
        }

        let mut packed = Vec::new();
        let mut byte_acc = 0u8;
        for (i, &bit) in bits.iter().enumerate() {
            byte_acc |= bit << (7 - (i % 8));
            if i % 8 == 7 {
                packed.push(byte_acc);
                byte_acc = 0;
            }
        }
        if bits.len() % 8 != 0 {
            packed.push(byte_acc);
        }

        let tree_bytes = self.serialized_tree.as_bytes();
        let mut output = Vec::new();
        let tree_len = tree_bytes.len() as u16;
        let bit_len = bits.len() as u16;

        output.extend_from_slice(&tree_len.to_le_bytes());
        output.extend_from_slice(&bit_len.to_le_bytes());
        output.extend_from_slice(tree_bytes);
        output.extend_from_slice(&packed);
        output
    }
}

fn mark_prefix_suffix(url: &str) -> String {
    let mut transformed = url.to_string();

    for &(pre, marker) in PREFIX_LIST {
        if transformed.starts_with(pre) {
            transformed = format!("{}{}{}", PREFIX_MARKER, marker, &transformed[pre.len()..]);
            break;
        }
    }

    for &(suf, marker) in SUFFIX_LIST {
        if transformed.ends_with(suf) {
            let idx = transformed.len() - suf.len();
            transformed = format!("{}{}{}", &transformed[..idx], SUFFIX_MARKER, marker);
            break;
        }
    }

    transformed
}

fn main() {
    let url = "http://localhost:3000";
    println!("Original URL: {}", url);

    let modified = mark_prefix_suffix(url);
    let compressor = UrlCompressor::from_input(&modified);
    let encoded_bytes = compressor.encode_bytes(url);

    println!("Encoded bytes: {:?}", encoded_bytes);
}
