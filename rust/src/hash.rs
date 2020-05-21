use std::marker::PhantomData;

pub use sha2::digest::Digest;
pub use sha2::Sha256;

use super::tree::TreeFold;

pub struct HashFold<H: Digest, B: AsRef<[u8]>> {
    _pd: PhantomData<(H, B)>,
}

impl<H: Digest, B: AsRef<[u8]>> TreeFold for HashFold<H, B> {
    type Leaf = B;
    type Target = Vec<u8>;
    type Error = std::convert::Infallible;

    fn input(leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
        Ok(H::digest(leaf.as_ref()).to_vec())
    }

    fn fold(a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
        let mut h = H::new();
        h.input(a);
        h.input(b);
        Ok(h.result().to_vec())
    }
}
