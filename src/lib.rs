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

    // sol_log(&raw_str);

    Ok(())
}
