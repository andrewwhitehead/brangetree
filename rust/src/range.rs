use crate::error::Error;
use crate::hash::{Digest, HashFold};
use crate::input::BitSink;
use crate::path::{Path, PathTracker};
use crate::tree::{TreeFold, TreeFolder};

#[inline]
pub fn make_range(left: u32, right: u32) -> [u8; 8] {
    let mut range = (right as u64).to_be_bytes();
    range[0..4].copy_from_slice(&left.to_be_bytes());
    range
}

pub fn range_hasher<H: Digest>() -> RangeTreeFolder<HashFold<H, [u8; 8]>> {
    RangeTreeFolder::new(HashFold::<H, [u8; 8]>::new())
}

pub fn range_path_hasher<H: Digest>(find_index: u32) -> RangePathTracker<HashFold<H, [u8; 8]>> {
    RangePathTracker::new(HashFold::<H, [u8; 8]>::new(), find_index)
}

pub trait RangeTarget {
    type Error;

    fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error>;
}

pub struct RangeTreeFolder<F: TreeFold<Leaf = [u8; 8]>> {
    pub folder: TreeFolder<F>,
}

impl<T: TreeFold<Leaf = [u8; 8]>> RangeTreeFolder<T> {
    pub fn new(base: T) -> Self {
        Self {
            folder: TreeFolder::new(base),
        }
    }

    pub fn fill(&mut self) -> usize {
        self.folder.fill(make_range(u32::MAX, u32::MAX)).unwrap()
    }

    pub fn len(&self) -> usize {
        self.folder.len()
    }

    pub fn result(self) -> Option<T::Target> {
        let (result, _) = self.folder.result().unwrap();
        result
    }

    pub fn complete(self) -> (Option<T::Target>, T) {
        self.folder.result().unwrap()
    }

    pub fn update_base<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.folder.update_base(f)
    }
}

impl<F: TreeFold<Leaf = [u8; 8]>> RangeTarget for RangeTreeFolder<F> {
    type Error = F::Error;

    fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error> {
        let range = make_range(left, right);
        self.folder.push(&range)
    }
}

pub struct RangePathTracker<T: TreeFold<Leaf = [u8; 8]>> {
    folder: RangeTreeFolder<PathTracker<T>>,
    find_index: u32,
    range: Option<(u32, u32)>,
}

impl<T: TreeFold<Leaf = [u8; 8]>> RangePathTracker<T> {
    pub fn new(base: T, find_index: u32) -> Self {
        Self {
            folder: RangeTreeFolder::new(PathTracker::new(base, None)),
            find_index,
            range: None,
        }
    }

    pub fn fill(&mut self) -> usize {
        self.folder.fill()
    }

    pub fn len(&self) -> usize {
        self.folder.len()
    }

    pub fn result(
        self,
    ) -> (
        Option<(u32, u32)>,
        Option<Path<T::Target>>,
        Option<T::Target>,
    ) {
        let (result, tracker) = self.folder.complete();
        (self.range, tracker.path_result(), result)
    }
}

impl<F: TreeFold<Leaf = [u8; 8]>> RangeTarget for RangePathTracker<F> {
    type Error = F::Error;

    fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error> {
        if self.find_index > left && self.find_index < right {
            self.folder.update_base(|b| b.track_next());
            self.range.replace((left, right));
        }
        self.folder.push_range(left, right)
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
            bit_idx: 1,
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

#[cfg(test)]
mod test {
    use super::*;

    struct RangeCollect {
        result: Vec<(u32, u32)>,
    }

    impl RangeCollect {
        pub fn new() -> Self {
            Self { result: vec![] }
        }
    }

    impl RangeTarget for RangeCollect {
        type Error = std::convert::Infallible;

        fn push_range(&mut self, left: u32, right: u32) -> Result<(), Self::Error> {
            self.result.push((left, right));
            Ok(())
        }
    }

    #[test]
    fn test_range() {
        let bits = &[true, false, false, true];
        let mut parser = RangeParser::new(RangeCollect::new());
        for bit in bits {
            parser.process_bits(*bit, 1).unwrap();
        }
        let collect = parser.complete().unwrap();
        assert_eq!(collect.result, vec![(0, 1), (1, 4), (4, u32::MAX)]);
    }
}
