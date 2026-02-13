use std::{
    env,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

fn main() {
    // Re-run build script if DLL changes
    println!("cargo:rerun-if-changed=../evil_dll/target/release/evil_dll.dll");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let input_path = manifest_dir
        .join("../evil_dll/target/release/evil_dll.dll");

    let stripped_path = out_dir.join("evil_dll.stripped.dll");
    let compressed_path = out_dir.join("evil_dll.dll.zst");

    // Copy original DLL
    fs::copy(&input_path, &stripped_path)
        .expect("Failed to copy DLL");

    // Strip symbols
    let status = Command::new("strip")
        .arg("--strip-unneeded")
        .arg(&stripped_path)
        .status()
        .expect("Failed to execute strip");

    if !status.success() {
        panic!("strip failed");
    }

    // Read stripped file
    let mut input_file = fs::File::open(&stripped_path)
        .expect("Failed to open stripped DLL");

    let mut buffer = Vec::new();
    input_file
        .read_to_end(&mut buffer)
        .expect("Failed to read DLL");

    // Compress with zstd
    let compressed = zstd::stream::encode_all(&buffer[..], 19)
        .expect("zstd compression failed");

    // Write compressed file
    let mut out_file = fs::File::create(&compressed_path)
        .expect("Failed to create output file");

    out_file
        .write_all(&compressed)
        .expect("Failed to write compressed DLL");

    println!("cargo:warning=Compressed DLL written to {:?}", compressed_path);
}
