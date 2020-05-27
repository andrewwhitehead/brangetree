use std::marker::PhantomData;

pub use sha2::digest::Digest;
pub use sha2::Sha256;

use super::tree::TreeFold;

pub struct HashFold<H: Digest, B: AsRef<[u8]>> {
    _pd: PhantomData<(H, B)>,
}

impl<H: Digest, B: AsRef<[u8]>> HashFold<H, B> {
    pub fn new() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<H: Digest, B: AsRef<[u8]>> TreeFold for HashFold<H, B> {
    type Leaf = B;
    type Target = Vec<u8>;
    type Error = std::convert::Infallible;

    fn input(&mut self, leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
        Ok(H::digest(leaf.as_ref()).to_vec())
    }

    fn fold(&mut self, a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
        let mut h = H::new();
        h.input(a);
        h.input(b);
        Ok(h.result().to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree::TreeFolder;

    #[test]
    fn test_hash() {
        let leaves: Vec<[u8; 8]> = [0, 1].iter().map(|n| (*n as u64).to_be_bytes()).collect();
        let (result, _) =
            TreeFolder::fold(HashFold::<Sha256, [u8; 8]>::new(), leaves.clone(), None).unwrap();
        let h0 = Sha256::digest(&leaves[0]).to_vec();
        let h1 = Sha256::digest(&leaves[1]).to_vec();
        let mut hasher = Sha256::new();
        hasher.input(h0);
        hasher.input(h1);
        let root = hasher.result().to_vec();
        assert_eq!(result.unwrap(), root);
    }
}
