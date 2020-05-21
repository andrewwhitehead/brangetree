use std::fs::File;
use std::io::prelude::*;

use flate2::read::GzDecoder;

use byte_slice_cast::*;

use crate::error::Error;

// could change to u32 depending on platform
type BitBlock = u64;

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

pub trait BitSink {
    type Result;

    fn process_bits(&mut self, revoked: bool, count: u32) -> Result<(), Error>;

    fn complete(self) -> Result<Self::Result, Error>;
}

pub fn process_zipped_bits<T>(path: String, proc: T) -> Result<T::Result, Error>
where
    T: BitSink,
{
    let bits = (std::mem::size_of::<BitBlock>() * 8) as u32;

    let target = fold_zipped_blocks(path, proc, |mut proc, block| {
        let mut size = block.len();
        let remain_size = size % 8;
        size -= remain_size;

        let elts = block[..size].as_slice_of::<BitBlock>().unwrap();
        for elt in elts {
            let elt = if cfg!(target_endian = "little") {
                elt.swap_bytes()
            } else {
                *elt
            };

            if elt == 0 || elt == u64::MAX {
                proc.process_bits(elt != 0, bits)?;
                continue;
            }

            for idx in (0..bits).rev() {
                proc.process_bits(elt >> idx & 1 != 0, 1)?;
            }
        }
        if remain_size > 0 {
            for elt in &block[size..] {
                for idx in (0..8).rev() {
                    proc.process_bits(elt >> idx & 1 != 0, 1)?;
                }
            }
        }

        Result::<_, Error>::Ok(proc)
    })?;

    let result = target.complete()?;
    Ok(result)
}
