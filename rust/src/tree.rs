use std::fmt;

pub trait TreeFold {
    type Leaf;
    type Target: Clone;
    type Error: fmt::Display;

    fn input(leaf: &Self::Leaf) -> Result<Self::Target, Self::Error>;

    fn fold(a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error>;
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

    #[allow(unused)]
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

    pub fn push(&mut self, leaf: &T::Leaf) -> Result<(), T::Error> {
        let mut h = T::input(leaf)?;
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
            self.push(&leaf)?;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::hash::{Digest, HashFold, Sha256};

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
    fn test_filled() {
        let leaves: Vec<String> = [0, 1, 2, 3, 4].iter().map(|n| n.to_string()).collect();
        let result = TreeFolder::<TestFold>::fold(leaves, Some("E".to_string()));
        assert_eq!(result.unwrap().unwrap(), "[[[0,1],[2,3]],[[4,E],[E,E]]]");
    }

    #[test]
    fn test_hash() {
        let leaves: Vec<[u8; 8]> = [0, 1].iter().map(|n| (*n as u64).to_be_bytes()).collect();
        let result = TreeFolder::<HashFold<Sha256, [u8; 8]>>::fold(leaves.clone(), None);
        let h0 = Sha256::digest(&leaves[0]).to_vec();
        let h1 = Sha256::digest(&leaves[1]).to_vec();
        let mut hasher = Sha256::new();
        hasher.input(h0);
        hasher.input(h1);
        let root = hasher.result().to_vec();
        assert_eq!(result.unwrap().unwrap(), root);
    }
}
