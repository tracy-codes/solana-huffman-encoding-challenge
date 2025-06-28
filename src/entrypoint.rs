use pinocchio::{
    entrypoint::InstructionContext, lazy_program_entrypoint, log::sol_log, no_allocator,
    nostd_panic_handler, ProgramResult,
};

use crate::instruction::{decoder::huffman_decode_url, HuffmanEncodingInstructionData};
lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[inline(always)]
fn process_instruction(context: InstructionContext) -> ProgramResult {
    let (_instruction_data, encoded_data) = HuffmanEncodingInstructionData::from_bytes(unsafe {
        context.instruction_data_unchecked()
    })?;
    let (decoded_len, decoded_bytes) = huffman_decode_url(encoded_data)?;

    // For validation - log the decoded URL (comment out for CU measurement)
    if let Ok(decoded_str) = core::str::from_utf8(&decoded_bytes[..decoded_len]) {
        sol_log(decoded_str);
    }

    Ok(())
}
