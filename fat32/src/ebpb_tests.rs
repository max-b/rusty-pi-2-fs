extern crate rand;

use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

use vfat::BiosParameterBlock;
use traits::*;
use tests;

#[test]
fn test_ebpb_data() {
    let mut ebpb1 = tests::resource!("ebpb1.img");
    let mut ebpb2 = tests::resource!("ebpb2.img");

    let mut data = [0u8; 1024];
    ebpb1.read_exact(&mut data[..512]).expect("read resource data");
    ebpb2.read_exact(&mut data[512..]).expect("read resource data");

    let ebpb1 = BiosParameterBlock::from(Cursor::new(&mut data[..]), 0).expect("valid EBPB");
    let ebpb2 = BiosParameterBlock::from(Cursor::new(&mut data[..]), 1).expect("valid EBPB");

    println!("ebpb1:\n{:#x?}", ebpb1);
    println!("ebpb2:\n{:#x?}", ebpb2);

    assert_eq!(std::str::from_utf8(&ebpb1.volume_label_string).unwrap(), "CS140E     ");
    assert_eq!(std::str::from_utf8(&ebpb2.volume_label_string).unwrap(), "NO NAME    ");

    assert_eq!(ebpb1.num_fats, 2);
    assert_eq!(ebpb2.num_fats, 2);

    assert_eq!(ebpb1.bytes_per_sector, 0x200);
    assert_eq!(ebpb2.bytes_per_sector, 0x400);
}
