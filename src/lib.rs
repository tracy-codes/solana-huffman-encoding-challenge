#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(not(feature = "no-entrypoint"))]
pinocchio_pubkey::declare_id!("ADUtWaDe3cn7V3oskWD7UWkdq9zxc6DcZKHoUH8vWBcD");

use pinocchio::{
    ProgramResult, entrypoint::InstructionContext, lazy_program_entrypoint, no_allocator,
    nostd_panic_handler,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[inline(always)]
fn process_instruction(context: InstructionContext) -> ProgramResult {
    let instruction_data = unsafe { context.instruction_data_unchecked() };

    let (_decoded_len, _decoded_bytes) = huffman_decode_url(instruction_data);
    // let res_str = unsafe {
    //     core::str::from_utf8_unchecked(_decoded_bytes.get_unchecked(.._decoded_len))
    // };
    // sol_log(&res_str);

    Ok(())
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Node {
    value: u8,
    left: u8,
    right: u8,
    is_leaf: bool,
}

impl Node {
    #[inline(always)]
    fn new_leaf(value: u8) -> Self {
        Self {
            is_leaf: true,
            value,
            left: 0,
            right: 0,
        }
    }

    #[inline(always)]
    fn new_internal(left: u8, right: u8) -> Self {
        Self {
            is_leaf: false,
            value: 0,
            left,
            right,
        }
    }
}

#[inline(always)]
fn huffman_decode_url(instruction_data: &[u8]) -> (usize, [u8; 256]) {
    let mut result = [0u8; 256];

    let original_len = unsafe { *instruction_data.get_unchecked(0) } as usize;

    let encoded_data = unsafe { instruction_data.get_unchecked(1..) };

    // Parse tree size from encoded data
    let tree_size = unsafe {
        u16::from_le_bytes([
            *encoded_data.get_unchecked(0),
            *encoded_data.get_unchecked(1),
        ])
    } as usize;

    let data_start = 2 + tree_size;

    // Build tree iteratively
    let mut nodes: [Node; 128] = [Node::new_leaf(0); 128];
    let mut node_count = 0u8;
    let root_idx = build_tree_iterative(
        unsafe { encoded_data.get_unchecked(2..2 + tree_size) },
        &mut nodes,
        &mut node_count,
    );

    // Decode bits with target length constraint
    let mut result_len = 0;
    let mut current_node = root_idx;
    let encoded_bits = unsafe { encoded_data.get_unchecked(data_start..) };

    for &byte in encoded_bits {
        for bit_offset in (0..8).rev() {
            if result_len >= original_len {
                break;
            }

            let bit = (byte >> bit_offset) & 1;
            let node = unsafe { *nodes.get_unchecked(current_node as usize) };

            if node.is_leaf {
                if result_len < result.len() {
                    unsafe {
                        *result.get_unchecked_mut(result_len) = node.value;
                    }
                    result_len += 1;
                }
                current_node = root_idx;

                // Process this bit with root if not done
                if result_len < original_len
                    && !unsafe { nodes.get_unchecked(root_idx as usize) }.is_leaf
                {
                    let root_node = unsafe { *nodes.get_unchecked(root_idx as usize) };
                    current_node = if bit == 0 {
                        root_node.left
                    } else {
                        root_node.right
                    };
                }
            } else {
                current_node = if bit == 0 { node.left } else { node.right };
            }
        }

        if result_len >= original_len {
            break;
        }
    }

    (result_len, result)
}

#[inline(always)]
fn build_tree_iterative(tree_data: &[u8], nodes: &mut [Node; 128], node_count: &mut u8) -> u8 {
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
                _stack_top -= 1; // Parent complete
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
                _stack_top -= 1; // Parent complete
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
