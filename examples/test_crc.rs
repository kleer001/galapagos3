use std::hash::Hasher;
use crc32fast::Hasher as Crc32Hasher;

fn main() {
    let mut hasher = Crc32Hasher::new();
    hasher.write(b"IHDR");
    let ihdr_data: Vec<u8> = vec![0, 1, 0, 0, 0, 1, 0, 0, 8, 6, 0, 0, 0];
    hasher.write(&ihdr_data);
    let crc = hasher.finish();
    
    println!("CRC type: {}", std::any::type_name_of_val(&crc));
    println!("CRC value: {}", crc);
    
    // Try different methods
    println!("\nUsing to_be_bytes():");
    let bytes = crc.to_be_bytes();
    println!("  Length: {}", bytes.len());
    
    println!("\nManual big-endian construction:");
    let manual = [(crc >> 24) & 0xff, (crc >> 16) & 0xff, (crc >> 8) & 0xff, crc & 0xff];
    for (i, b) in manual.iter().enumerate() {
        println!("  byte[{}]: 0x{:02x}", i, b);
    }
    
    // Test on a known value
    let test_val: u32 = 0x12345678;
    println!("\nTest with 0x12345678:");
    println!("  to_be_bytes(): {:?}", test_val.to_be_bytes());
    println!("  Expected:      [0x12, 0x34, 0x56, 0x78]");
}
