use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::time::Instant;

use byte_slice_cast::*;

use flate2::read::GzDecoder;

use naturalize::to_natural;

use sha2::{digest::Digest, Sha256};

fn natural_sort<I: IntoIterator<Item = String>>(paths: I) -> Vec<String> {
    let mut paths: Vec<(String, String)> = paths
        .into_iter()
        .map(|arg| (to_natural(&arg).unwrap(), arg))
        .collect();
    paths.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    paths.into_iter().map(|a| a.1).collect()
}

type T = u64;

pub trait TreeFold {
    type Leaf;
    type Target: Clone;
    type Error: fmt::Display;

    fn input(leaf: &Self::Leaf) -> Result<Self::Target, Self::Error>;

    fn fold(a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error>;
}

pub struct RangeHash<H: Digest> {
    _pd: PhantomData<H>,
}

impl<H: Digest> TreeFold for RangeHash<H> {
    type Leaf = [u8; 8];
    type Target = Vec<u8>;
    type Error = String;

    fn input(leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
        Ok(H::digest(leaf).to_vec())
    }

    fn fold(a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
        let mut h = H::new();
        h.input(a);
        h.input(b);
        Ok(h.result().to_vec())
    }
}

pub struct TreeFolder<T: TreeFold> {
    stack: Vec<T::Target>,
    leaf_count: usize,
}

impl<T: TreeFold> TreeFolder<T> {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            leaf_count: 0,
        }
    }

    pub fn fold<L>(leaves: L, fill_input: Option<T::Leaf>) -> Result<Option<T::Target>, T::Error>
    where
        L: IntoIterator<Item = T::Leaf>,
    {
        let mut inst = Self::new();
        inst.extend(leaves)?;
        if let Some(fill_input) = fill_input {
            inst.fill(fill_input)?;
        }
        inst.result()
    }

    pub fn push(&mut self, leaf: T::Leaf) -> Result<(), T::Error> {
        let mut h = T::input(&leaf)?;
        let mut b = self.leaf_count + 1;
        while b & 1 == 0 {
            h = T::fold(&self.stack.pop().unwrap(), &h)?;
            b = b >> 1;
        }
        self.stack.push(h);
        self.leaf_count += 1;
        Ok(())
    }

    pub fn extend<L>(&mut self, leaves: L) -> Result<(), T::Error>
    where
        L: IntoIterator<Item = T::Leaf>,
    {
        for leaf in leaves.into_iter() {
            self.push(leaf)?;
        }
        Ok(())
    }

    pub fn fill(&mut self, fill_input: T::Leaf) -> Result<usize, T::Error> {
        let mut fill_cache: Vec<T::Target> = vec![];

        let mut filler = |depth: usize| -> Result<T::Target, T::Error> {
            let mut d = fill_cache.len();
            if d > depth {
                return Ok(fill_cache[depth].clone());
            }
            let mut h = if d == 0 {
                T::input(&fill_input)?
            } else {
                let prev = fill_cache[d - 1].clone();
                T::fold(&prev, &prev)?
            };
            loop {
                fill_cache.push(h.clone());
                d += 1;
                if d > depth {
                    break;
                }
                h = T::fold(&h, &h)?;
            }
            Ok(h)
        };

        let leaf_count = self.leaf_count;
        let fill_size = leaf_count.next_power_of_two();
        let mut fill_count = fill_size - leaf_count;
        let mut fill_depth: isize = 0;
        let mut leaf_count_filled = leaf_count;

        while fill_count > 0 {
            if fill_count & 1 != 0 {
                let mut h = filler(fill_depth as usize)?;
                leaf_count_filled += 1 << fill_depth;
                let mut b = leaf_count_filled;
                let mut c: isize = 0;
                while b & 1 == 0 {
                    b = b >> 1;
                    c += 1;
                }
                c -= fill_depth;
                while c > 0 {
                    h = T::fold(&self.stack.pop().unwrap(), &h)?;
                    c -= 1;
                }
                self.stack.push(h);
            }
            fill_depth += 1;
            fill_count = fill_count >> 1;
        }
        self.leaf_count = leaf_count_filled;
        Ok(fill_count)
    }

    pub fn result(mut self) -> Result<Option<T::Target>, T::Error> {
        if self.stack.is_empty() {
            return Ok(None);
        }
        let mut root = self.stack.pop().unwrap();
        while !self.stack.is_empty() {
            root = T::fold(&self.stack.pop().unwrap(), &root)?;
        }
        Ok(Some(root))
    }

    pub fn len(&self) -> usize {
        self.leaf_count
    }
}

struct ReadIter<R: Read> {
    buf: Vec<u8>,
    source: R,
}

impl<'a, R: Read> ReadIter<R> {
    pub fn new(source: R, bufsize: usize) -> Self {
        Self {
            buf: vec![0u8; bufsize],
            source,
        }
    }

    fn try_fold<B, F, E>(&mut self, init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, &[u8]) -> Result<B, E>,
        E: From<std::io::Error>,
    {
        let mut acc = init;
        loop {
            match self.source.read(self.buf.as_mut_slice()) {
                Ok(0) => return Ok(acc),
                Ok(sz) => {
                    acc = f(acc, &self.buf[0..sz])?;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

fn fold_zipped_blocks<B, F, E>(path: String, init: B, f: F) -> Result<B, E>
where
    F: FnMut(B, &[u8]) -> Result<B, E>,
    E: From<std::io::Error>,
{
    let fp = File::open(path)?;
    let mut reader = ReadIter::new(GzDecoder::new(fp), 1024);
    reader.try_fold(init, f)
}

#[inline]
fn make_range(left: u32, right: u32) -> [u8; 8] {
    let mut range = (right as u64).to_be_bytes();
    range[0..4].copy_from_slice(&left.to_be_bytes());
    range
}

fn main() -> std::io::Result<()> {
    let paths: Vec<String> = natural_sort(env::args().skip(1));
    let bits = std::mem::size_of::<T>() * 8;
    for path in paths {
        let start = Instant::now();
        let mut left: u32 = 0;
        let mut in_rev: bool = false;
        let mut bit_idx: u32 = 0;
        let mut leaf_idx: u32 = 0;

        let mut folder = fold_zipped_blocks(
            path.clone(),
            TreeFolder::<RangeHash<Sha256>>::new(),
            |mut folder, block| {
                // FIXME check uneven block length (mod 8)
                let mut size = block.len();
                let remain_size = size % 8;
                size -= remain_size;

                let elts = block[..size].as_slice_of::<T>().unwrap();
                for elt in elts {
                    let elt = if cfg!(target_endian = "little") {
                        elt.swap_bytes()
                    } else {
                        *elt
                    };

                    for idx in (0..bits).rev() {
                        let revoked = elt >> idx & 1 != 0;
                        if revoked {
                            if !in_rev {
                                // FIXME - convert error
                                folder.push(make_range(left, bit_idx)).unwrap();
                                leaf_idx += 1;
                                in_rev = true;
                            }
                            left = bit_idx;
                        } else {
                            in_rev = false;
                        }
                        bit_idx += 1;
                    }
                }
                if remain_size > 0 {
                    //println!("remain {}", remain_size);
                }

                std::io::Result::Ok(folder)
            },
        )?;

        // FIXME convert error
        folder.push(make_range(left, u32::MAX)).unwrap();
        leaf_idx += 1;
        folder.fill(make_range(u32::MAX, u32::MAX)).unwrap();
        let leaf_filled_idx = folder.len();
        let result = folder.result().unwrap();
        let dur = Instant::now() - start;

        if let Some(root) = result {
            println!(
                "{} {} {} {} {}",
                path,
                leaf_filled_idx,
                leaf_idx,
                hex::encode(root),
                dur.as_secs_f64()
            );
        } else {
            println!("{} no hash produced", path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestFold;

    impl TreeFold for TestFold {
        type Leaf = String;
        type Target = String;
        type Error = String;

        fn input(leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
            Ok(leaf.clone())
        }

        fn fold(a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
            Ok(format!("[{},{}]", a, b))
        }
    }

    #[test]
    fn test_basic() {
        let leaves: Vec<String> = [0, 1, 2, 3, 4].iter().map(|n| n.to_string()).collect();
        let result = TreeFolder::<TestFold>::fold(leaves, None);
        assert_eq!(result.unwrap().unwrap(), "[[[0,1],[2,3]],4]");
    }

    #[test]
    fn test_fill() {
        let leaves: Vec<String> = [0, 1, 2, 3, 4].iter().map(|n| n.to_string()).collect();
        let result = TreeFolder::<TestFold>::fold(leaves, Some("E".to_string()));
        assert_eq!(result.unwrap().unwrap(), "[[[0,1],[2,3]],[[4,E],[E,E]]]");
    }

    #[test]
    fn test_hash() {
        let leaves: Vec<[u8; 8]> = [0, 1].iter().map(|n| (*n as u64).to_be_bytes()).collect();
        let result = TreeFolder::<RangeHash<Sha256>>::fold(leaves.clone(), None);
        let h0 = Sha256::digest(&leaves[0]).to_vec();
        let h1 = Sha256::digest(&leaves[1]).to_vec();
        let mut hasher = Sha256::new();
        hasher.input(h0);
        hasher.input(h1);
        let root = hasher.result().to_vec();
        assert_eq!(result.unwrap().unwrap(), root);
    }
}
