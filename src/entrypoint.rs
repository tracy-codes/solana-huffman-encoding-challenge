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

    let (_decoded_len, _decoded_bytes) = unsafe {huffman_decode_url(instruction_data)};

    // For validation - uncomment to log decoded URL (comment out for CU measurement)
    // let res_str = unsafe {
    //     core::str::from_utf8_unchecked(_decoded_bytes.get_unchecked(0..))
    // };
    // sol_log(&res_str);

    Ok(())
}
