use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Instruction: Decode");

    let decoded = decode_url(instruction_data)?;
    msg!("Decoded: {}", core::str::from_utf8(&decoded).unwrap_or("<invalid utf8>"));

    Ok(())
}

fn decode_url(input: &[u8]) -> Result<Vec<u8>, ProgramError> {
    let mut result = Vec::with_capacity(128);

    let mut i = 0;
    while i < input.len() {
        match input[i] {
            0x01 => result.extend_from_slice(b"https://"),
            0x02 => result.extend_from_slice(b"http://"),
            0x03 => result.extend_from_slice(b".com"),
            0x04 => result.extend_from_slice(b".in"),
            0x05 => result.extend_from_slice(b".net"),
            0x06 => result.extend_from_slice(b".org"),
            0x07 => result.extend_from_slice(b".git"),
            0x08 => result.extend_from_slice(b".dev"),
            byte => result.push(byte),
        }
        i += 1;
    }

    Ok(result)
}
