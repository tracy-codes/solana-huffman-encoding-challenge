// This is the encoder program

use std::collections::HashMap;
pub fn encode_url_to_instruction_data(input: &str) -> Result<Vec<u8>, String> {
    let mut table: HashMap<&str, u8> = HashMap::new();
    table.insert("https://", 0x01);
    table.insert("http://", 0x02);
    table.insert(".com",     0x03);
    table.insert(".in",      0x04);
    table.insert(".net",     0x05);
    table.insert(".org",     0x06);
    table.insert(".git",     0x07);
    table.insert(".dev",     0x08);

    let mut output: Vec<u8> = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&_ch) = chars.peek() {
        let remaining: String = chars.clone().collect();

        // Match known substrings
        let mut matched = false;
        for (key, &token) in &table {
            if remaining.starts_with(key) {
                output.push(token);
                for _ in key.chars() {
                    chars.next();
                }
                matched = true;
                break;
            }
        }

        if matched {
            continue;
        }

        // Fallback: push character's UTF-8 bytes directly
        let ch = chars.next().unwrap();
        let mut buf = [0; 4];
        let utf8 = ch.encode_utf8(&mut buf);
        output.extend_from_slice(utf8.as_bytes());
    }

    Ok(output)
}


fn main() {
    let url = "https://🦝👀🍹🌏.net";
    match encode_url_to_instruction_data(url) {
        Ok(bytes) => {
            println!("Encoded Instruction Data (hex): {}", hex::encode(&bytes));
        }
        Err(e) => {
            eprintln!("Encoding error: {}", e);
        }
    }
}
