use pinocchio::program_error::ProgramError;
pub mod decoder;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HuffmanEncodingInstructionData {
    pub encoded_len: u32,
}

impl HuffmanEncodingInstructionData {
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), ProgramError> {
        if bytes.len() < 4 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let encoded_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let encoded_start = 4;
        let encoded_end = encoded_start + encoded_len as usize;

        if bytes.len() < encoded_end {
            return Err(ProgramError::InvalidInstructionData);
        }

        let encoded_data = &bytes[encoded_start..encoded_end];
        Ok((Self { encoded_len }, encoded_data))
    }
}
