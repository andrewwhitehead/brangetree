use std::env;
use std::time::Instant;

use generic_array::typenum::U2;
use lazy_static::lazy_static;
use naturalize::to_natural;
use neptune::poseidon::{HashMode, Poseidon, PoseidonConstants};
use neptune::scalar_from_u64;
use paired::bls12_381::{Bls12, Fr};

use brangetree::{process_zipped_bits, Error, RangeParser, RangeTreeFolder, TreeFold};

lazy_static! {
    static ref CONSTANTS: PoseidonConstants<Bls12, U2> = PoseidonConstants::new();
}

pub struct PHashFold<'a> {
    hasher: Poseidon<'a, Bls12>,
}

impl PHashFold<'_> {
    pub fn new() -> Self {
        Self {
            hasher: Poseidon::new(&CONSTANTS),
        }
    }
}

impl TreeFold for PHashFold<'_> {
    type Leaf = [u8; 8];
    type Target = Fr;
    type Error = std::convert::Infallible;

    fn input(&mut self, leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
        Ok(scalar_from_u64(u64::from_be_bytes(*leaf)))
    }

    fn fold(&mut self, a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
        self.hasher.reset();
        self.hasher.input(*a).unwrap();
        self.hasher.input(*b).unwrap();
        Ok(self.hasher.hash_in_mode(HashMode::OptimizedStatic))
    }
}

pub struct PHashResult {
    pub leaf_count: usize,
    pub filled_count: usize,
    pub root: Option<Fr>,
}

pub fn hash_zipped(path: String, fill: bool) -> Result<PHashResult, Error> {
    let target = RangeTreeFolder::new(PHashFold::new());
    let mut parsed = process_zipped_bits(path, RangeParser::new(target))?;
    let leaf_count = parsed.len();
    let filled_count = if fill {
        parsed.fill();
        parsed.len()
    } else {
        leaf_count
    };
    let root = parsed.result();
    Ok(PHashResult {
        leaf_count,
        filled_count,
        root,
    })
}

fn natural_sort<I: IntoIterator<Item = String>>(paths: I) -> Vec<String> {
    let mut paths: Vec<(String, String)> = paths
        .into_iter()
        .map(|arg| (to_natural(&arg).unwrap(), arg))
        .collect();
    paths.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    paths.into_iter().map(|a| a.1).collect()
}

fn main() -> Result<(), Error> {
    let paths: Vec<String> = natural_sort(env::args().skip(1));
    for path in paths {
        let start = Instant::now();
        let mut result = hash_zipped(path.clone(), true)?;
        let dur = Instant::now() - start;

        if let Some(root) = result.root.take() {
            println!(
                "{} {} {} {} {:0.3}",
                path,
                result.filled_count,
                result.leaf_count,
                root,
                dur.as_secs_f64()
            );
        } else {
            println!("{} no hash produced", path);
        }
    }
    Ok(())
}
