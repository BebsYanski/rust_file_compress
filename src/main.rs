//! Simple file-to-file gzip compressor.
//!
//! This binary reads a source file from disk and writes a gzipped version to a
//! target path using [`flate2`].
//!
//! ## Usage
//! `compress <source> <target>`
//!
//! ## Notes
//! - This is a streaming implementation: it does **not** load the whole file
//!   into memory.
//! - The output file is only guaranteed to be a valid gzip stream after
//!   calling [`flate2::write::GzEncoder::finish`].

use flate2::Compression;
use flate2::write::GzEncoder;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::copy;
use std::time::Instant;

fn main() {
    // Keep `main` tiny: we can document and test logic more easily in `run()`.
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Compress `source` into `target` as a gzip stream.
///
/// Exits with a non-zero status code if:
/// - arguments are missing/invalid
/// - the source file can't be opened
/// - the target file can't be created
/// - compression or finalization fails
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut argv = env::args();
    let program = argv.next().unwrap_or_else(|| "compress".to_owned());

    let source = argv
        .next()
        .ok_or_else(|| format!("usage: {program} <source> <target>"))?;
    let target = argv
        .next()
        .ok_or_else(|| format!("usage: {program} <source> <target>"))?;

    // Buffering reduces syscall overhead when copying from disk.
    let mut input = BufReader::new(File::open(&source)?);

    // `GzEncoder` implements `Write`: we stream bytes into it and it produces a
    // gzip stream into the underlying `File`.
    let output = File::create(&target)?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    let start = Instant::now();

    // Stream the input into the encoder. `copy` returns the number of bytes
    // read from `input` (uncompressed bytes).
    let _uncompressed_bytes = copy(&mut input, &mut encoder)?;

    // Important: `finish()` writes the gzip trailer and returns the underlying
    // writer. Without this, the output may be incomplete/corrupt.
    let output = encoder.finish()?;

    println!("Source: {source}");
    println!("Target: {target}");
    println!("Source len: {}", input.get_ref().metadata()?.len());
    println!("Target len: {}", output.metadata()?.len());
    println!("Elapsed Time: {:?}", start.elapsed());

    Ok(())
}
