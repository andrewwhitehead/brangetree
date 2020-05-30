mod error;
mod hash;
mod input;
mod path;
mod range;
mod tree;

pub use error::Error;
pub use hash::Digest;
pub use input::process_zipped_bits;
pub use path::{Path, PathJoin};
pub use range::{range_hasher, range_path_hasher, RangeParser, RangePathTracker, RangeTreeFolder};
pub use tree::{TreeFold, TreeFolder};

pub struct HashResult {
    pub leaf_count: usize,
    pub filled_count: usize,
    pub root: Option<Vec<u8>>,
}

pub type HashPath = Path<Vec<u8>>;

pub fn hash_zipped<H: Digest>(path: String, fill: bool) -> Result<HashResult, Error> {
    let target = range_hasher::<H>();
    let mut parsed = process_zipped_bits(path, RangeParser::new(target))?;
    let leaf_count = parsed.len();
    let filled_count = if fill {
        parsed.fill();
        parsed.len()
    } else {
        leaf_count
    };
    let root = parsed.result();
    Ok(HashResult {
        leaf_count,
        filled_count,
        root,
    })
}

// test method exercising PathTracker
pub fn find_merkle_path<H: Digest>(
    path: String,
    index: u32,
) -> Result<(Option<(u32, u32)>, Option<HashPath>, HashResult), Error> {
    let target = range_path_hasher::<H>(index);
    let mut parsed = process_zipped_bits(path, RangeParser::new(target))?;
    let leaf_count = parsed.len();
    parsed.fill();
    let filled_count = parsed.len();
    let (range, path, root) = parsed.result();
    Ok((
        range,
        path,
        HashResult {
            leaf_count,
            filled_count,
            root,
        },
    ))
}
