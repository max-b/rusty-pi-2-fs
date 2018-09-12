#![feature(decl_macro)]
#![feature(extern_prelude)]
#![allow(safe_packed_borrows)]

#[cfg(not(target_endian="little"))]
compile_error!("only little endian platforms supported");

extern crate byteorder;

#[cfg(test)]
#[macro_use]
mod tests;

#[cfg(test)]
mod mbr_tests;

#[cfg(test)]
mod ebpb_tests;

mod mbr;
mod util;

pub mod vfat;
pub mod traits;

pub use mbr::*;
