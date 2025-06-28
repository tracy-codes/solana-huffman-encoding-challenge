use mollusk_svm::result::{Check, ProgramResult};
use mollusk_svm::Mollusk;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;
pub mod encoder;

use crate::encoder::huffman_encode_url;

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

#[test]
pub fn test_all_challenge_urls() {
    let mollusk = mollusk();
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

    for test_url in test_urls {
        let encoded_url = huffman_encode_url(test_url);
        let instruction_data = create_instruction_data(&encoded_url);

        let ix_accounts = vec![];

        let ix = Instruction::new_with_bytes(PROGRAM, &instruction_data, ix_accounts.clone());
        let tx_accounts = &vec![];

        let result =
            mollusk.process_and_validate_instruction(&ix, tx_accounts, &[Check::success()]);
        assert_eq!(result.program_result, ProgramResult::Success);
    }
}
