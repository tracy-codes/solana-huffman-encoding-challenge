use mollusk_svm::Mollusk;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;
pub mod encoder;
use std::fs::File;
use std::io::Write;

use crate::encoder::huffman_encode_url;

pub const PROGRAM: Pubkey = Pubkey::new_from_array(solana_huffman_encoding_challenge::ID);

pub fn mollusk() -> Mollusk {
    Mollusk::new(&PROGRAM, "target/deploy/solana_huffman_encoding_challenge")
}

pub fn create_instruction_data(encoded_url: &[u8], original_size: u32) -> Vec<u8> {
    let mut data = Vec::new();

    // Format: [original_len: 1][tree_size: 2][tree_data][encoded_bits]
    data.push(original_size as u8); // Single byte for original length

    // Extract tree size from encoded data (first 4 bytes as u32, convert to u16)
    let tree_size = u16::from_le_bytes([encoded_url[0], encoded_url[1]]);

    // data.extend_from_slice(&tree_size.to_le_bytes()); // u16 tree size
    data.push(tree_size as u8); // Single byte for tree size (assuming <= 255)
    data.extend_from_slice(&encoded_url[2..]); // Rest of encoded data (tree + bits)
    data
}

#[test]
pub fn test_all_challenge_urls_and_store_metrics() {
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

    // Prepare CSV header
    let mut report = String::new();
    report.push_str(
        "URL,Original Size (bytes),Compressed Size (bytes),Compression Ratio,CU Consumed\n",
    );

    for test_url in test_urls {
        let original_size = test_url.len();
        let encoded_url = huffman_encode_url(test_url);
        let compressed_size = encoded_url.len();
        let compression_ratio = (original_size as f64) / (compressed_size as f64);

        let instruction_data = create_instruction_data(&encoded_url, test_url.len() as u32);
        let ix_accounts = vec![];
        let ix = solana_sdk::instruction::Instruction::new_with_bytes(
            PROGRAM,
            &instruction_data,
            ix_accounts.clone(),
        );
        let tx_accounts = &vec![];

        let result = mollusk.process_and_validate_instruction(
            &ix,
            tx_accounts,
            &[mollusk_svm::result::Check::success()],
        );
        assert_eq!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        );

        let cu_consumed = result.compute_units_consumed;

        // Escape commas if necessary and add the record to our report.
        let record = format!(
            "{},{},{},{:.2}x,{}\n",
            test_url, original_size, compressed_size, compression_ratio, cu_consumed
        );
        report.push_str(&record);
    }

    // Store the report in the target directory.
    // Adjust the path as needed.
    let report_path = "target/url_metrics.csv";
    let mut file = File::create(report_path).expect("failed to create url_metrics.csv file");
    file.write_all(report.as_bytes())
        .expect("failed to write metrics to file");
}
