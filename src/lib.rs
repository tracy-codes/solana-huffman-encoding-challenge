#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod instruction;
pinocchio_pubkey::declare_id!("G7FuQezcCopF4815BYXMiwsiLKdw3vexXyL42BXjZhg");
