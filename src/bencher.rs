pub const BENCH_COUNT: usize = 128;
pub const BUFFER_LENGTH: usize = 4096;

use crate::hasher::{Blake2bHasher, FileHasher, OUTPUT_LENGTH};
use crate::RunOptions;
use hex::ToHex;
use kdam::{tqdm, BarExt, Column, RichProgress};
use std::fs::metadata;
use std::io::Result;
use std::time::Instant;

pub(crate) struct Blake2bBencher;

impl Blake2bBencher {
    const TITLE_RUST_CRYPTO: &'static str = "Blake2b256 - RustCrypto";
    const TITLE_RFC: &'static str = "Blake2b256 - RFC";
    const TITLE_SIMD: &'static str = "Blake2b256 - SIMD";

    fn get_file_size(options: &RunOptions) -> Result<usize> {
        let file_metadata = metadata(&options.filepath)?;

        Ok(file_metadata.len() as usize)
    }

    fn get_progress_bar(file_size: usize, test_name: &str) -> RichProgress {
        let mut pb = RichProgress::new(
            tqdm!(
                total = BENCH_COUNT * file_size,
                unit_scale = true,
                unit_divisor = 1024,
                unit = "B"
            ),
            vec![
                Column::Spinner(
                    "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().map(|x| x.to_string()).collect::<Vec<String>>(),
                    80.0,
                    1.0,
                ),
                Column::text(&format!("[bold blue]{test_name}")),
                Column::Bar,
                Column::Percentage(1),
                Column::text("•"),
                Column::CountTotal,
                Column::text("•"),
                Column::Rate,
                Column::text("•"),
                Column::RemainingTime,
            ],
        );

        pb.refresh();

        pb
    }

    fn bench(title: &str, run_options: &RunOptions, hasher: Blake2bHasher) -> Result<()> {
        let file_size = Self::get_file_size(run_options)?;
        let test_file_path = &run_options.filepath;
        let mut progress_bar = Self::get_progress_bar(file_size, title);
        let mut output = [0; OUTPUT_LENGTH];
        let instant = Instant::now();
        let mut bytes_read = 0;

        for _ in 0..BENCH_COUNT {
            let mut file_hasher = FileHasher::<BUFFER_LENGTH>::new(test_file_path, hasher.clone())?;
            bytes_read += file_hasher.update_hash_buffered()?;
            file_hasher.get_hash(&mut output);
            progress_bar.update_to(bytes_read);
        }

        let seconds_per_operation = instant.elapsed().as_secs_f64();
        let bytes_per_operation = (file_size * BENCH_COUNT) as f64 / seconds_per_operation;
        let result = output.encode_hex::<String>();
        println!("\n** Result     => {}", result);
        println!("** Throughput => {:.2} MB/s", bytes_per_operation / (1024.0 * 1024.0));

        Ok(())
    }

    pub(crate) fn bench_rust_crypto(options: &RunOptions) -> Result<()> {
        Self::bench(Self::TITLE_RUST_CRYPTO, options, Blake2bHasher::new_rust_crypto())?;

        Ok(())
    }

    pub(crate) fn bench_rfc(options: &RunOptions) -> Result<()> {
        Self::bench(Self::TITLE_RFC, options, Blake2bHasher::new_rfc())?;

        Ok(())
    }

    pub(crate) fn bench_simd(options: &RunOptions) -> Result<()> {
        Self::bench(Self::TITLE_SIMD, options, Blake2bHasher::new_simd())?;

        Ok(())
    }
}
