use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
};

const MAX_TREE_LEN: usize = 512;
const MAX_OUTPUT_LEN: usize = 256;

const PREFIX_MARKER: char = '\u{00F1}'; // ñ
const SUFFIX_MARKER: char = '\u{00F2}'; // ò

const PREFIXES: &[(&str, char)] = &[
    ("https://www.", '1'),
    ("https://", '2'),
    ("https://localhost", '3'),
    ("http://", '4'),
    ("http://localhost", '5'),
];

const SUFFIXES: &[(&str, char)] = &[
    (".com", '1'),
    (".org", '2'),
    (".net", '3'),
    ("/index.html", '4'),
    (".git", '5'),
];

#[derive(Clone)]
struct FlatNode {
    is_leaf: bool,
    symbol: char,
    left: usize,
    right: usize,
}

entrypoint!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if data.len() < 4 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let tree_len = u16::from_le_bytes([data[0], data[1]]) as usize;
    let bit_len  = u16::from_le_bytes([data[2], data[3]]) as usize;

    if 4 + tree_len > data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let tree_bytes = &data[4..4 + tree_len];
    let bit_data = &data[4 + tree_len..];

    let tree_str = std::str::from_utf8(tree_bytes)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let tree_chars: Vec<char> = tree_str.chars().collect();
    let mut idx = 0;
    let mut flat: Vec<FlatNode> = Vec::with_capacity(MAX_TREE_LEN);

    fn parse(tree: &[char], idx: &mut usize, flat: &mut Vec<FlatNode>) -> Result<usize, ProgramError> {
        if *idx >= tree.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let tag = tree[*idx];
        *idx += 1;

        if tag == '1' {
            if *idx >= tree.len() {
                return Err(ProgramError::InvalidInstructionData);
            }
            let ch = tree[*idx];
            *idx += 1;
            let i = flat.len();
            flat.push(FlatNode { is_leaf: true, symbol: ch, left: 0, right: 0 });
            Ok(i)
        } else if tag == '0' {
            let l = parse(tree, idx, flat)?;
            let r = parse(tree, idx, flat)?;
            let i = flat.len();
            flat.push(FlatNode { is_leaf: false, symbol: '\0', left: l, right: r });
            Ok(i)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    let root = parse(&tree_chars, &mut idx, &mut flat)?;

    let mut decoded = [0u8; MAX_OUTPUT_LEN];
    let mut out_len = 0;
    let mut total_bits = 0;
    let mut node = root;

    for &byte in bit_data {
        for i in (0..8).rev() {
            if total_bits >= bit_len { break; }
            total_bits += 1;

            let bit = (byte >> i) & 1;
            node = if bit == 0 {
                flat[node].left
            } else {
                flat[node].right
            };

            if flat[node].is_leaf {
                let ch = flat[node].symbol;
                let mut buf = [0u8; 4];
                let ch_bytes = ch.encode_utf8(&mut buf).as_bytes();

                if out_len + ch_bytes.len() > MAX_OUTPUT_LEN {
                    return Err(ProgramError::InvalidInstructionData);
                }

                decoded[out_len..out_len + ch_bytes.len()].copy_from_slice(ch_bytes);
                out_len += ch_bytes.len();
                node = root;
            }
        }
    }

    // Attempt final push if ended on a leaf
    if total_bits == bit_len && flat[node].is_leaf {
        let ch = flat[node].symbol;
        let mut buf = [0u8; 4];
        let ch_bytes = ch.encode_utf8(&mut buf).as_bytes();
        if out_len + ch_bytes.len() <= MAX_OUTPUT_LEN {
            decoded[out_len..out_len + ch_bytes.len()].copy_from_slice(ch_bytes);
            out_len += ch_bytes.len();
        }
    }

    let raw_str = std::str::from_utf8(&decoded[..out_len])
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let restored = restore_url(raw_str);
    msg!("✅ Decoded URL: {}", restored);

    Ok(())
}

fn restore_url(input: &str) -> String {
    let mut chars = input.chars().peekable();
    let mut output = String::new();

    while let Some(ch) = chars.next() {
        if ch == PREFIX_MARKER {
            if let Some(code) = chars.next() {
                if let Some(&(prefix, _)) = PREFIXES.iter().find(|&&(_, c)| c == code) {
                    output.push_str(prefix);
                }
            }
        } else if ch == SUFFIX_MARKER {
            if let Some(code) = chars.next() {
                if let Some(&(suffix, _)) = SUFFIXES.iter().find(|&&(_, c)| c == code) {
                    output.push_str(suffix);
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}
