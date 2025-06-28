use pinocchio::program_error::ProgramError;

const MAX_TREE_SIZE: usize = 128;
const MAX_OUTPUT_SIZE: usize = 256;

#[derive(Clone, Copy)]
struct TreeNode {
    is_leaf: bool,
    byte_value: u8,
    left_child: u8,  // Index to left child (if internal node)
    right_child: u8, // Index to right child (if internal node)
}

impl TreeNode {
    fn new_leaf(byte: u8) -> Self {
        Self {
            is_leaf: true,
            byte_value: byte,
            left_child: 0,
            right_child: 0,
        }
    }

    fn new_internal(left: u8, right: u8) -> Self {
        Self {
            is_leaf: false,
            byte_value: 0,
            left_child: left,
            right_child: right,
        }
    }
}

pub struct HuffmanDecoder {
    nodes: [TreeNode; MAX_TREE_SIZE],
    node_count: u8,
    root: u8,
}

impl HuffmanDecoder {
    pub fn new() -> Self {
        Self {
            nodes: [TreeNode::new_leaf(0); MAX_TREE_SIZE],
            node_count: 0,
            root: 0,
        }
    }

    pub fn load_tree(&mut self, tree_data: &[u8]) -> Result<usize, ProgramError> {
        self.node_count = 0;
        if tree_data.len() < 4 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let tree_size =
            u32::from_le_bytes([tree_data[0], tree_data[1], tree_data[2], tree_data[3]]) as usize;

        if tree_data.len() < 4 + tree_size {
            return Err(ProgramError::InvalidInstructionData);
        }

        let pos = 4;
        self.root = self.deserialize_node(&tree_data[pos..pos + tree_size], &mut 0)?;

        Ok(4 + tree_size)
    }

    fn deserialize_node(&mut self, data: &[u8], pos: &mut usize) -> Result<u8, ProgramError> {
        if *pos >= data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let node_type = data[*pos];
        *pos += 1;

        if node_type == 1 {
            // Leaf node
            if *pos >= data.len() {
                return Err(ProgramError::InvalidInstructionData);
            }
            let byte_value = data[*pos];
            *pos += 1;

            let node_index = self.node_count;
            if (node_index as usize) >= MAX_TREE_SIZE {
                return Err(ProgramError::AccountDataTooSmall);
            }

            self.nodes[node_index as usize] = TreeNode::new_leaf(byte_value);
            self.node_count += 1;
            Ok(node_index)
        } else {
            // Internal node
            let left_child = self.deserialize_node(data, pos)?;
            let right_child = self.deserialize_node(data, pos)?;

            let node_index = self.node_count;
            if (node_index as usize) >= MAX_TREE_SIZE {
                return Err(ProgramError::AccountDataTooSmall);
            }

            self.nodes[node_index as usize] = TreeNode::new_internal(left_child, right_child);
            self.node_count += 1;
            Ok(node_index)
        }
    }

    pub fn decode(
        &self,
        encoded_data: &[u8],
        max_output_len: usize,
    ) -> Result<(usize, [u8; MAX_OUTPUT_SIZE]), ProgramError> {
        let mut result = [0u8; MAX_OUTPUT_SIZE];
        let mut result_len = 0;
        let mut current_node = self.root;

        let actual_max_len = max_output_len.min(MAX_OUTPUT_SIZE);

        for &byte in encoded_data {
            for bit_pos in (0..8).rev() {
                if result_len >= actual_max_len {
                    break;
                }

                let bit = (byte >> bit_pos) & 1;

                // First traverse based on the bit (only if not at leaf)
                let node = &self.nodes[current_node as usize];
                if !node.is_leaf {
                    current_node = if bit == 0 {
                        node.left_child
                    } else {
                        node.right_child
                    };
                }

                // Then check if we've reached a leaf
                let node = &self.nodes[current_node as usize];
                if node.is_leaf {
                    result[result_len] = node.byte_value;
                    result_len += 1;
                    current_node = self.root; // Reset to root for next character

                    if result_len >= actual_max_len {
                        break;
                    }
                }
            }

            if result_len >= actual_max_len {
                break;
            }
        }

        Ok((result_len, result))
    }
}

pub fn huffman_decode_url(
    encoded_data: &[u8],
) -> Result<(usize, [u8; MAX_OUTPUT_SIZE]), ProgramError> {
    let mut decoder = HuffmanDecoder::new();
    let tree_end = decoder.load_tree(encoded_data)?;
    let data_start = tree_end;

    if data_start >= encoded_data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    decoder.decode(&encoded_data[data_start..], MAX_OUTPUT_SIZE)
}
