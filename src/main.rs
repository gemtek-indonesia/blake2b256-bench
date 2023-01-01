use digest::consts::U32;
use digest::generic_array::GenericArray;
use digest::Digest;
use hex::ToHex;
use kdam::{tqdm, BarExt, Column, RichProgress};
use std::fs::metadata;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;

const BENCH_COUNT: usize = 128;
const BUFFER_LENGTH: usize = 4096;
const OUTPUT_LENGTH: usize = 32;

pub struct ChunkedFileReader<const BUFFER_LENGTH: usize> {
    buffer: [u8; BUFFER_LENGTH],
    file: File,
}

impl<const BUFFER_LENGTH: usize> ChunkedFileReader<BUFFER_LENGTH> {
    pub fn new<AnyPath: AsRef<Path>>(path: AnyPath) -> anyhow::Result<Self> {
        let buffer = [0; BUFFER_LENGTH];
        let file = OpenOptions::new().read(true).write(false).open(path)?;

        Ok(Self { buffer, file })
    }

    pub fn update_hash_on_read<UpdateHashFunc: FnMut(&[u8])>(
        mut self,
        mut callback: UpdateHashFunc,
    ) -> anyhow::Result<usize> {
        let mut bytes_read = 0;

        while let Ok(read_length) = self.file.read(&mut self.buffer) {
            if read_length == 0 {
                break;
            }

            callback(&self.buffer[..read_length]);
            bytes_read += read_length;
        }

        Ok(bytes_read)
    }
}

pub enum Blake2bHasher {
    RustCrypto(blake2::Blake2b<U32>),
    Rfc(blake2_rfc::blake2b::Blake2b),
    Simd(blake2b_simd::State),
}

impl Blake2bHasher {
    pub fn new_rust_crypto() -> Self {
        let hasher = blake2::Blake2b::<U32>::new();

        Self::RustCrypto(hasher)
    }

    pub fn new_rfc() -> Self {
        let hasher = blake2_rfc::blake2b::Blake2b::new(32);

        Self::Rfc(hasher)
    }

    pub fn new_simd() -> Self {
        let mut hasher_params = blake2b_simd::Params::default();
        hasher_params.hash_length(32);
        let hasher = hasher_params.to_state();

        Self::Simd(hasher)
    }

    pub fn update(&mut self, data: &[u8]) {
        match self {
            Self::RustCrypto(hasher) => hasher.update(data),
            Self::Rfc(hasher) => hasher.update(data),
            Self::Simd(hasher) => {
                hasher.update(data);
            }
        }
    }

    pub fn finalize_into(self, output: &mut [u8]) {
        match self {
            Self::RustCrypto(hasher) => {
                hasher.finalize_into(GenericArray::from_mut_slice(output));
            }
            Self::Rfc(hasher) => {
                let result = hasher.finalize();
                output.copy_from_slice(result.as_bytes());
            }
            Self::Simd(hasher) => {
                let result = hasher.finalize();
                output.copy_from_slice(result.as_bytes());
            }
        }
    }
}

fn bench<HasherFactory, AnyPath>(
    file_size: usize,
    mut progress_bar: RichProgress,
    hasher_factory: HasherFactory,
    test_file_path: AnyPath,
) -> anyhow::Result<(String, f64)>
where
    HasherFactory: Fn() -> Blake2bHasher,
    AnyPath: AsRef<Path>,
{
    let mut output = [0; OUTPUT_LENGTH];
    let instant = Instant::now();
    let mut bytes_read = 0;
    let path_str = test_file_path.as_ref();

    for _ in 0..BENCH_COUNT {
        let reader = ChunkedFileReader::<BUFFER_LENGTH>::new(path_str)?;
        let mut hasher = hasher_factory();
        bytes_read += reader.update_hash_on_read(|chunk| {
            hasher.update(chunk);
        })?;
        hasher.finalize_into(&mut output);
        progress_bar.update_to(bytes_read);
    }

    let seconds_per_operation = instant.elapsed().as_secs_f64();
    let bytes_per_operation =
        (file_size * BENCH_COUNT) as f64 / seconds_per_operation;
    let result = output.encode_hex();

    Ok((result, bytes_per_operation))
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
                "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
                    .chars()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
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

fn bench_with_progress_bar<HasherFactory, AnyPath>(
    title: &str,
    file_size: usize,
    hasher_factory: HasherFactory,
    test_file_path: AnyPath,
) -> anyhow::Result<()>
where
    HasherFactory: Fn() -> Blake2bHasher,
    AnyPath: AsRef<Path>,
{
    let pb = get_progress_bar(file_size, title);
    let (output, throughput) =
        bench(file_size, pb, hasher_factory, test_file_path)?;

    println!("\n** Result     => {}", output);
    println!(
        "** Throughput => {:.2} MB/s",
        throughput / (1024.0 * 1024.0)
    );

    Ok(())
}

#[derive(StructOpt)]
#[structopt(about = "Blake2b 256-bit Hashing Benchmark")]
struct RunOptions {
    /// file to hash
    #[structopt(short, long, parse(from_os_str))]
    filepath: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = RunOptions::from_args();
    let file_metadata = metadata(&opt.filepath)?;
    let file_size = file_metadata.len() as usize;

    bench_with_progress_bar(
        "Blake2b256 - RustCrypto",
        file_size,
        Blake2bHasher::new_rust_crypto,
        &opt.filepath,
    )?;
    bench_with_progress_bar(
        "Blake2b256 - RFC",
        file_size,
        Blake2bHasher::new_rfc,
        &opt.filepath,
    )?;
    bench_with_progress_bar(
        "Blake2b256 - SIMD",
        file_size,
        Blake2bHasher::new_simd,
        &opt.filepath,
    )?;

    Ok(())
}
