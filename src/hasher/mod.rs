pub mod rfc;
pub mod rust_crypto;
pub mod simd;

pub const OUTPUT_LENGTH: usize = 32;

use enum_dispatch::enum_dispatch;
use rfc::Rfc;
use rust_crypto::RustCrypto;
use simd::Simd;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Result;
use std::path::Path;

#[enum_dispatch]
pub trait SlicedHashUpdate: Clone {
    fn update_with_slice(&mut self, data: &[u8]);
    fn finalize_into(self, output: &mut [u8]);
}

#[enum_dispatch(SlicedHashUpdate)]
#[derive(Clone)]
pub enum Blake2bHasher {
    RustCrypto,
    Rfc,
    Simd,
}

impl Blake2bHasher {
    pub fn new_rfc() -> Blake2bHasher {
        Blake2bHasher::Rfc(Default::default())
    }

    pub fn new_rust_crypto() -> Blake2bHasher {
        Blake2bHasher::RustCrypto(Default::default())
    }

    pub fn new_simd() -> Blake2bHasher {
        Blake2bHasher::Simd(Default::default())
    }
}

pub struct FileHasher<const BUFFER_LENGTH: usize> {
    file: File,
    hasher: Blake2bHasher,
    buffer: [u8; BUFFER_LENGTH],
}

impl<const BUFFER_LENGTH: usize> FileHasher<BUFFER_LENGTH> {
    pub fn new<AnyPath: AsRef<Path>>(path: AnyPath, hasher: Blake2bHasher) -> Result<Self> {
        let buffer = [0; BUFFER_LENGTH];
        let file = OpenOptions::new().read(true).write(false).open(path)?;

        Ok(Self { file, hasher, buffer })
    }

    pub fn update_hash_buffered(&mut self) -> Result<usize> {
        let mut bytes_read = 0;
        let buffer_reference = &mut self.buffer;

        while let Ok(read_length) = self.file.read(buffer_reference) {
            if read_length == 0 {
                break;
            }

            self.hasher.update_with_slice(&buffer_reference[..read_length]);
            bytes_read += read_length;
        }

        Ok(bytes_read)
    }

    pub fn get_hash(self, output: &mut [u8]) {
        self.hasher.finalize_into(output);
    }
}
