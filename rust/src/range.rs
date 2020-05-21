use crate::error::Error;
use crate::hash::{Digest, HashFold};
use crate::input::BitSink;
use crate::tree::TreeFolder;

#[inline]
fn make_range(left: u32, right: u32) -> [u8; 8] {
    let mut range = (right as u64).to_be_bytes();
    range[0..4].copy_from_slice(&left.to_be_bytes());
    range
}

pub trait RangeTarget {
    type Error: std::error::Error;

    fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error>;
}

pub struct RangeHash<H: Digest> {
    pub folder: TreeFolder<HashFold<H, [u8; 8]>>,
}

impl<H: Digest> RangeHash<H> {
    pub fn new() -> Self {
        Self {
            folder: TreeFolder::<HashFold<H, [u8; 8]>>::new(),
        }
    }

    pub fn fill(&mut self) -> usize {
        self.folder.fill(make_range(u32::MAX, u32::MAX)).unwrap()
    }

    pub fn len(&self) -> usize {
        self.folder.len()
    }

    pub fn result(self) -> Option<Vec<u8>> {
        self.folder.result().unwrap()
    }
}

impl<H: Digest> RangeTarget for RangeHash<H> {
    type Error = std::convert::Infallible;

    fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error> {
        let range = make_range(left, right);
        self.folder.push(&range)
    }
}

pub struct RangeParser<T: RangeTarget> {
    pub left: u32,
    pub in_rev: bool,
    pub bit_idx: u32,
    pub target: T,
}

impl<T: RangeTarget> RangeParser<T> {
    pub fn new(target: T) -> Self {
        Self {
            left: 0,
            in_rev: false,
            bit_idx: 0,
            target,
        }
    }
}

impl<T: RangeTarget> BitSink for RangeParser<T>
where
    Error: From<T::Error>,
{
    type Result = T;

    fn process_bits(&mut self, revoked: bool, count: u32) -> Result<(), Error> {
        if revoked {
            if !self.in_rev {
                self.target.push_range(self.left, self.bit_idx)?;
                self.in_rev = true;
            }
            self.left = self.bit_idx;
        } else {
            self.in_rev = false;
        }
        self.bit_idx += count;
        Ok(())
    }

    fn complete(mut self) -> Result<Self::Result, Error> {
        self.target.push_range(self.left, u32::MAX)?;
        Ok(self.target)
    }
}
