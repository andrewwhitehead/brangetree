pub trait TreeFold {
    type Leaf;
    type Target: Clone;
    type Error: std::fmt::Debug;

    fn input(&mut self, leaf: &Self::Leaf) -> Result<Self::Target, Self::Error>;

    fn fold(&mut self, a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error>;

    fn start_fill(&mut self) {}
    fn end_fill(&mut self) {}
}

pub struct TreeFolder<T: TreeFold> {
    base: T,
    stack: Vec<T::Target>,
    leaf_count: usize,
}

impl<T: TreeFold> TreeFolder<T> {
    pub fn new(base: T) -> Self {
        Self {
            base,
            stack: vec![],
            leaf_count: 0,
        }
    }

    #[allow(unused)]
    pub fn fold<L>(
        base: T,
        leaves: L,
        fill_input: Option<T::Leaf>,
    ) -> Result<(Option<T::Target>, T), T::Error>
    where
        L: IntoIterator<Item = T::Leaf>,
    {
        let mut inst = Self::new(base);
        inst.extend(leaves)?;
        if let Some(fill_input) = fill_input {
            inst.fill(fill_input)?;
        }
        inst.result()
    }

    pub fn push(&mut self, leaf: &T::Leaf) -> Result<(), T::Error> {
        let mut h = self.base.input(leaf)?;
        let mut b = self.leaf_count + 1;
        while b & 1 == 0 {
            let left = self.stack.pop().unwrap();
            h = self.base.fold(&left, &h)?;
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

        let mut filler = |depth: usize, base: &mut T| -> Result<T::Target, T::Error> {
            let mut d = fill_cache.len();
            if d > depth {
                return Ok(fill_cache[depth].clone());
            }
            let mut h = if d == 0 {
                base.input(&fill_input)?
            } else {
                let prev = fill_cache[d - 1].clone();
                base.fold(&prev, &prev)?
            };
            loop {
                fill_cache.push(h.clone());
                d += 1;
                if d > depth {
                    break;
                }
                h = base.fold(&h, &h)?;
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
                self.base.start_fill();
                let mut h = filler(fill_depth as usize, &mut self.base)?;
                self.base.end_fill();
                leaf_count_filled += 1 << fill_depth;
                let mut b = leaf_count_filled;
                let mut c: isize = 0;
                while b & 1 == 0 {
                    b = b >> 1;
                    c += 1;
                }
                c -= fill_depth;
                while c > 0 {
                    let left = self.stack.pop().unwrap();
                    h = self.base.fold(&left, &h)?;
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

    pub fn result(mut self) -> Result<(Option<T::Target>, T), T::Error> {
        let result = if self.stack.is_empty() {
            None
        } else if self.stack.len() > 1 {
            let mut root = self.stack.pop().unwrap();
            while !self.stack.is_empty() {
                root = self.base.fold(&self.stack.pop().unwrap(), &root)?;
            }
            self.stack.push(root.clone());
            Some(root)
        } else {
            Some(self.stack[0].clone())
        };
        Ok((result, self.base))
    }

    pub fn len(&self) -> usize {
        self.leaf_count
    }

    pub fn update_base<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.base)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub struct TestFold;

    impl TreeFold for TestFold {
        type Leaf = String;
        type Target = String;
        type Error = String;

        fn input(&mut self, leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
            Ok(leaf.clone())
        }

        fn fold(
            &mut self,
            a: &Self::Target,
            b: &Self::Target,
        ) -> Result<Self::Target, Self::Error> {
            Ok(format!("[{},{}]", a, b))
        }
    }

    #[test]
    fn test_unfilled() {
        let leaves = (0..=4).map(|n| n.to_string());
        let (result, _) = TreeFolder::fold(TestFold {}, leaves, None).unwrap();
        assert_eq!(result.unwrap(), "[[[0,1],[2,3]],4]");
    }

    #[test]
    fn test_filled_even() {
        let leaves = (0..4).map(|n| n.to_string());
        let (result, _) = TreeFolder::fold(TestFold {}, leaves, Some("E".to_string())).unwrap();
        assert_eq!(result.unwrap(), "[[0,1],[2,3]]");
    }

    #[test]
    fn test_filled_uneven() {
        let leaves = (0..=4).map(|n| n.to_string());
        let (result, _) = TreeFolder::fold(TestFold {}, leaves, Some("E".to_string())).unwrap();
        assert_eq!(result.unwrap(), "[[[0,1],[2,3]],[[4,E],[E,E]]]");
    }
}
