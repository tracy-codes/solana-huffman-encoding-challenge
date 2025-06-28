use pinocchio::{
    entrypoint::InstructionContext, lazy_program_entrypoint, log::sol_log, no_allocator,
    nostd_panic_handler, ProgramResult,
};

use crate::instruction::decoder::huffman_decode_url;

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[inline(always)]
fn process_instruction(context: InstructionContext) -> ProgramResult {
    let instruction_data = unsafe { context.instruction_data_unchecked() };

    // Decode directly from instruction data
    let (_decoded_len, _decoded_bytes) = huffman_decode_url(instruction_data)?;

    // For validation - uncomment to log decoded URL (comment out for CU measurement)
    // if let Ok(decoded_str) = core::str::from_utf8(&_decoded_bytes[.._decoded_len]) {
    //     sol_log(decoded_str);
    // }

    Ok(())
}
