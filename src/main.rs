#[cfg(feature = "tlsmalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod bencher;
mod hasher;

use crate::bencher::Blake2bBencher;
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Blake2b 256-bit Hashing Benchmark")]
struct RunOptions {
    /// file to hash
    #[structopt(short, long, parse(from_os_str))]
    filepath: PathBuf,
}

fn main() -> Result<()> {
    let opt = RunOptions::from_args();
    Blake2bBencher::bench_rust_crypto(&opt)?;
    Blake2bBencher::bench_rfc(&opt)?;
    Blake2bBencher::bench_simd(&opt)?;

    Ok(())
}
