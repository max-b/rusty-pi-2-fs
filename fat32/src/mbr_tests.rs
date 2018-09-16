extern crate rand;

use std::io::prelude::*;
use std::io::Cursor;

use mbr::MasterBootRecord;
use tests;

#[test]
fn test_mbr_data() {
    let mut mbr = tests::resource!("mbr.img");
    let mut data = [0u8; 512];
    mbr.read_exact(&mut data).expect("read resource data");
    let _mbr_record = MasterBootRecord::from(&mut Cursor::new(&mut data[..])).expect("valid MBR");
}
