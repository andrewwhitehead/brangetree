mod error;
mod hash;
mod input;
mod range;
mod tree;

pub use error::Error;
pub use hash::{Digest, Sha256};
use input::process_zipped_bits;
use range::{RangeHash, RangeParser};

pub struct HashResult {
    pub leaf_idx: usize,
    pub leaf_filled_idx: usize,
    pub root: Option<Vec<u8>>,
}

pub fn hash_zipped<H: Digest>(path: String, fill: bool) -> Result<HashResult, Error> {
    let target = RangeHash::<H>::new();
    let mut parsed = process_zipped_bits(path, RangeParser::new(target))?;
    let leaf_idx = parsed.len();
    if fill {
        parsed.fill();
    }
    let leaf_filled_idx = parsed.len();
    let root = parsed.result();
    Ok(HashResult {
        leaf_idx,
        leaf_filled_idx,
        root,
    })
}
