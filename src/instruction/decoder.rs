#[derive(Clone, Copy)]
#[repr(C)]
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

#[inline(always)]
pub unsafe fn huffman_decode_url(instruction_data: &[u8]) -> (usize, [u8; 128]) {
    let mut result = [0u8; 128];

    // Format: [original_len: 1][tree_size: 2][tree_data][encoded_bits]
    let original_len = *instruction_data.get_unchecked(0) as usize;

    // let tree_size = u16::from_le_bytes([
    //     *instruction_data.get_unchecked(1),
    //     *instruction_data.get_unchecked(2),
    // ]) as usize;
    let tree_size = *instruction_data.get_unchecked(1) as usize; // Assuming tree size is <= 255

    let data_start = 2 + tree_size;

    // Build tree iteratively
    let mut nodes: [Node; 64] = [Node::new_leaf(0); 64];
    let mut node_count = 0u8;
    let root_idx = build_tree_iterative(
        instruction_data.get_unchecked(2..2 + tree_size),
        &mut nodes,
        &mut node_count,
    );

    // Optimized decoding loop
    let mut result_len = 0;
    let mut current_node = root_idx;
    let encoded_bits = instruction_data.get_unchecked(data_start..);

    for &byte in encoded_bits {
        if result_len >= original_len {
            break;
        }

        // Unroll bit processing for better performance
        let mut bits = byte;
        for _ in 0..8 {
            if result_len >= original_len {
                break;
            }

            let bit = (bits >> 7) & 1;
            bits <<= 1;

            let node = *nodes.get_unchecked(current_node as usize);

            if node.is_leaf {
                *result.get_unchecked_mut(result_len) = node.byte_value;
                result_len += 1;
                current_node = root_idx;

                // Inline root processing
                if result_len < original_len {
                    let root_node = *nodes.get_unchecked(root_idx as usize);
                    if !root_node.is_leaf {
                        current_node = if bit == 0 {
                            root_node.left
                        } else {
                            root_node.right
                        };
                    }
                }
            } else {
                current_node = if bit == 0 { node.left } else { node.right };
            }
        }
    }

    (result_len, result)
}

#[inline(always)]
fn build_tree_iterative(tree_data: &[u8], nodes: &mut [Node; 64], node_count: &mut u8) -> u8 {
    let mut pos = 0;
    let mut stack: [u8; 16] = [0; 16];
    let mut _stack_top = 0;

    // Read first node
    let node_type = unsafe { *tree_data.get_unchecked(pos) };
    pos += 1;

    let root_idx = *node_count;

    if node_type == 1 {
        // Single leaf node case
        unsafe {
            *nodes.get_unchecked_mut(*node_count as usize) =
                Node::new_leaf(*tree_data.get_unchecked(pos));
        }
        *node_count += 1;
        return root_idx;
    } else {
        // Internal root node
        unsafe {
            *nodes.get_unchecked_mut(*node_count as usize) = Node::new_internal(0, 0);
            *stack.get_unchecked_mut(0) = *node_count;
        }
        _stack_top = 1;
        *node_count += 1;
    }

    // Process remaining nodes
    while pos < tree_data.len() && _stack_top > 0 {
        let node_type = unsafe { *tree_data.get_unchecked(pos) };
        pos += 1;
        let current_idx = *node_count;

        if node_type == 1 {
            // Leaf node
            unsafe {
                *nodes.get_unchecked_mut(current_idx as usize) =
                    Node::new_leaf(*tree_data.get_unchecked(pos));
            }
            pos += 1;
            *node_count += 1;

            // Attach to parent
            let parent_idx = unsafe { *stack.get_unchecked(_stack_top - 1) };
            let parent = unsafe { nodes.get_unchecked_mut(parent_idx as usize) };

            if parent.left == 0 {
                parent.left = current_idx;
            } else {
                parent.right = current_idx;
                _stack_top -= 1;
            }
        } else {
            // Internal node
            unsafe {
                *nodes.get_unchecked_mut(current_idx as usize) = Node::new_internal(0, 0);
            }
            *node_count += 1;

            // Attach to parent
            let parent_idx = unsafe { *stack.get_unchecked(_stack_top - 1) };
            let parent = unsafe { nodes.get_unchecked_mut(parent_idx as usize) };

            if parent.left == 0 {
                parent.left = current_idx;
            } else {
                parent.right = current_idx;
                _stack_top -= 1;
            }

            // Push to stack
            unsafe {
                *stack.get_unchecked_mut(_stack_top) = current_idx;
            }
            _stack_top += 1;
        }
    }

    root_idx
}
