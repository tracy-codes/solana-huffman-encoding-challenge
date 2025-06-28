#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(not(feature = "no-entrypoint"))]

pinocchio_pubkey::declare_id!("ADUtWaDe3cn7V3oskWD7UWkdq9zxc6DcZKHoUH8vWBcD");

use pinocchio::{
    ProgramResult, entrypoint::InstructionContext, lazy_program_entrypoint, log::sol_log,
    no_allocator, nostd_panic_handler, program_error::ProgramError,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

const MAX_TREE_LEN: usize = 128;
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

#[derive(Clone, Copy)]
struct FlatNode {
    is_leaf: bool,
    symbol: char,
    left: usize,
    right: usize,
}

#[inline(always)]
fn process_instruction(context: InstructionContext) -> ProgramResult {
    let data = unsafe { context.instruction_data_unchecked() };

    if data.len() < 4 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let tree_len = u16::from_le_bytes([data[0], data[1]]) as usize;
    let bit_len = u16::from_le_bytes([data[2], data[3]]) as usize;

    if 4 + tree_len > data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let tree_bytes = &data[4..4 + tree_len];
    let bit_data = &data[4 + tree_len..];

    let tree_str =
        core::str::from_utf8(tree_bytes).map_err(|_| ProgramError::InvalidInstructionData)?;

    let mut flat = [FlatNode {
        is_leaf: false,
        symbol: '\0',
        left: 0,
        right: 0,
    }; MAX_TREE_LEN];
    let mut flat_len = 0;

    fn parse<I: Iterator<Item = char>>(
        iter: &mut I,
        flat: &mut [FlatNode; MAX_TREE_LEN],
        flat_len: &mut usize,
    ) -> Result<usize, ProgramError> {
        let tag = iter.next().ok_or(ProgramError::InvalidInstructionData)?;

        if tag == '1' {
            let symbol = iter.next().ok_or(ProgramError::InvalidInstructionData)?;
            let i = *flat_len;
            if i >= MAX_TREE_LEN {
                return Err(ProgramError::AccountDataTooSmall);
            }
            flat[i] = FlatNode {
                is_leaf: true,
                symbol,
                left: 0,
                right: 0,
            };
            *flat_len += 1;
            Ok(i)
        } else if tag == '0' {
            let l = parse(iter, flat, flat_len)?;
            let r = parse(iter, flat, flat_len)?;
            let i = *flat_len;
            if i >= MAX_TREE_LEN {
                return Err(ProgramError::AccountDataTooSmall);
            }
            flat[i] = FlatNode {
                is_leaf: false,
                symbol: '\0',
                left: l,
                right: r,
            };
            *flat_len += 1;
            Ok(i)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    let mut tree_iter = tree_str.chars();
    let root = parse(&mut tree_iter, &mut flat, &mut flat_len)?;

    let mut decoded = [0u8; MAX_OUTPUT_LEN];
    let mut out_len = 0;
    let mut total_bits = 0;
    let mut node = root;

    for &byte in bit_data {
        for i in (0..8).rev() {
            if total_bits >= bit_len {
                break;
            }
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

    let raw_str = core::str::from_utf8(&decoded[..out_len])
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    restore_url(raw_str)?;

    // let (out_len, restored_buf) = restore_url(raw_str)?;
    // let restored_str = core::str::from_utf8(&restored_buf[..out_len])
    //     .map_err(|_| ProgramError::InvalidInstructionData)?;
    // sol_log(&raw_str);

    Ok(())
}

fn restore_url(input: &str) -> Result<(usize, [u8; MAX_OUTPUT_LEN]), ProgramError> {
    let mut chars = input.chars().peekable();
    let mut output = [0u8; MAX_OUTPUT_LEN];
    let mut out_len = 0;

    while let Some(ch) = chars.next() {
        if ch == PREFIX_MARKER {
            if let Some(code) = chars.next() {
                if let Some(&(prefix, _)) = PREFIXES.iter().find(|&&(_, c)| c == code) {
                    let bytes = prefix.as_bytes();
                    if out_len + bytes.len() > MAX_OUTPUT_LEN {
                        return Err(ProgramError::InvalidInstructionData);
                    }
                    output[out_len..out_len + bytes.len()].copy_from_slice(bytes);
                    out_len += bytes.len();
                }
            }
        } else if ch == SUFFIX_MARKER {
            if let Some(code) = chars.next() {
                if let Some(&(suffix, _)) = SUFFIXES.iter().find(|&&(_, c)| c == code) {
                    let bytes = suffix.as_bytes();
                    if out_len + bytes.len() > MAX_OUTPUT_LEN {
                        return Err(ProgramError::InvalidInstructionData);
                    }
                    output[out_len..out_len + bytes.len()].copy_from_slice(bytes);
                    out_len += bytes.len();
                }
            }
        } else {
            let mut buf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buf).as_bytes();
            if out_len + encoded.len() > MAX_OUTPUT_LEN {
                return Err(ProgramError::InvalidInstructionData);
            }
            output[out_len..out_len + encoded.len()].copy_from_slice(encoded);
            out_len += encoded.len();
        }
    }

    Ok((out_len, output))
}
