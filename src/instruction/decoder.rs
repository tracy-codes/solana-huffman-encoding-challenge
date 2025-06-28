use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy)]
struct Node {
    is_leaf: bool,
    byte_value: u8,
    left: u8,
    right: u8,
}

impl Node {
    #[inline(always)]
    fn new_leaf(byte_value: u8) -> Self {
        Self {
            is_leaf: true,
            byte_value,
            left: 0,
            right: 0,
        }
    }

    #[inline(always)]
    fn new_internal(left: u8, right: u8) -> Self {
        Self {
            is_leaf: false,
            byte_value: 0,
            left,
            right,
        }
    }
}

pub fn huffman_decode_url(instruction_data: &[u8]) -> Result<(usize, [u8; 256]), ProgramError> {
    let mut result = [0u8; 256];

    // Parse instruction data: [original_len: 4][encoded_len: 4][encoded_data: encoded_len]
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let original_len = u32::from_le_bytes([
        instruction_data[0],
        instruction_data[1],
        instruction_data[2],
        instruction_data[3],
    ]) as usize;

    let encoded_len = u32::from_le_bytes([
        instruction_data[4],
        instruction_data[5],
        instruction_data[6],
        instruction_data[7],
    ]) as usize;

    if original_len > 256 || instruction_data.len() < 8 + encoded_len {
        return Err(ProgramError::InvalidInstructionData);
    }

    let encoded_data = &instruction_data[8..8 + encoded_len];

    if encoded_data.len() < 4 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Parse tree size from encoded data
    let tree_size = u32::from_le_bytes([
        encoded_data[0],
        encoded_data[1],
        encoded_data[2],
        encoded_data[3],
    ]) as usize;

    let data_start = 4 + tree_size;

    if data_start >= encoded_data.len() || tree_size == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Build tree iteratively
    let mut nodes: [Node; 127] = [Node::new_leaf(0); 127];
    let mut node_count = 0u8;
    let root_idx =
        build_tree_iterative(&encoded_data[4..4 + tree_size], &mut nodes, &mut node_count)?;

    if node_count == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Decode bits with target length constraint
    let mut result_len = 0;
    let mut current_node = root_idx;
    let encoded_bits = &encoded_data[data_start..];

    for &byte in encoded_bits {
        for bit_offset in (0..8).rev() {
            if result_len >= original_len {
                break;
            }

            let bit = (byte >> bit_offset) & 1;
            let node = nodes[current_node as usize];

            if node.is_leaf {
                if result_len < result.len() {
                    result[result_len] = node.byte_value;
                    result_len += 1;
                }
                current_node = root_idx;

                // Process this bit with root if not done
                if result_len < original_len && !nodes[root_idx as usize].is_leaf {
                    current_node = if bit == 0 {
                        nodes[root_idx as usize].left
                    } else {
                        nodes[root_idx as usize].right
                    };
                }
            } else {
                current_node = if bit == 0 { node.left } else { node.right };

                if current_node >= node_count {
                    return Err(ProgramError::InvalidInstructionData);
                }
            }
        }

        if result_len >= original_len {
            break;
        }
    }

    Ok((result_len, result))
}

#[inline(always)]
fn build_tree_iterative(
    tree_data: &[u8],
    nodes: &mut [Node; 127],
    node_count: &mut u8,
) -> Result<u8, ProgramError> {
    if tree_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut pos = 0;
    let mut stack: [u8; 32] = [0; 32];
    let mut _stack_top = 0;

    // Read first node
    if pos >= tree_data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let node_type = tree_data[pos];
    pos += 1;

    let root_idx = *node_count;

    if node_type == 1 {
        // Single leaf node case
        if pos >= tree_data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let byte_value = tree_data[pos];
        if *node_count >= 127 {
            return Err(ProgramError::InvalidInstructionData);
        }
        nodes[*node_count as usize] = Node::new_leaf(byte_value);
        *node_count += 1;
        return Ok(root_idx);
    } else {
        // Internal root node
        if *node_count >= 127 {
            return Err(ProgramError::InvalidInstructionData);
        }
        nodes[*node_count as usize] = Node::new_internal(0, 0);
        stack[0] = *node_count;
        _stack_top = 1;
        *node_count += 1;
    }

    // Process remaining nodes
    while pos < tree_data.len() && _stack_top > 0 {
        let node_type = tree_data[pos];
        pos += 1;

        if *node_count >= 127 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let current_idx = *node_count;

        if node_type == 1 {
            // Leaf node
            if pos >= tree_data.len() {
                return Err(ProgramError::InvalidInstructionData);
            }
            let byte_value = tree_data[pos];
            pos += 1;

            nodes[current_idx as usize] = Node::new_leaf(byte_value);
            *node_count += 1;

            // Attach to parent
            let parent_idx = stack[_stack_top - 1];
            let parent = &mut nodes[parent_idx as usize];

            if parent.left == 0 {
                parent.left = current_idx;
            } else {
                parent.right = current_idx;
                _stack_top -= 1; // Parent complete
            }
        } else {
            // Internal node
            nodes[current_idx as usize] = Node::new_internal(0, 0);
            *node_count += 1;

            // Attach to parent
            let parent_idx = stack[_stack_top - 1];
            let parent = &mut nodes[parent_idx as usize];

            if parent.left == 0 {
                parent.left = current_idx;
            } else {
                parent.right = current_idx;
                _stack_top -= 1; // Parent complete
            }

            // Push to stack
            if _stack_top < 32 {
                stack[_stack_top] = current_idx;
                _stack_top += 1;
            } else {
                return Err(ProgramError::InvalidInstructionData);
            }
        }
    }

    Ok(root_idx)
}
