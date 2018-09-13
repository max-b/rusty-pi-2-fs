extern crate rand;

use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

use mbr::{MasterBootRecord, PartitionEntry, CHS};
use tests;
use traits::*;

#[test]
fn test_mbr_data() {
    let mut mbr = tests::resource!("mbr.img");
    let mut data = [0u8; 512];
    mbr.read_exact(&mut data).expect("read resource data");
    let mbr_record = MasterBootRecord::from(&mut Cursor::new(&mut data[..])).expect("valid MBR");

    println!("mbr:\n{:#x?}", mbr_record);
}
