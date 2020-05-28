# brangetree - Binary Range Tree

Support for creating binary Merkle tree hashes from a list of sorted leaf nodes, at high speed and
with minimal memory consumption.

## Generating test data

The `scripts/gen-data.py` utility can be used to generate test data. Each data file is a gzipped
series of bits, where 0 bits represent non-revoked credentials and 1 bits represent revoked
credentials. The test data spreads the revocations over the entire range (assuming every
credential has been issued) which represents close to worst case performance for the particular
registry size.

## Python utilities

From the `python` directory, `scripts/brt-hash.py` can be used to produce the hash for a number
of data files provided as arguments to the script. Similarly, `scripts/inspect-data.py` can be
used to provide statistics about a particular data file.

## Rust utilities

From the `rust` directory, resources can be built using `cargo build --release`. There are
currently two executables: `brt-hash` hashes and prints statistics for a set of data files,
while `brt-find` can be used to obtain the hash chain for a particular (non-revoked)
credential index. These executables may be run using (for example):
`cargo run --release --bin brt-hash -- ../data/22bits_*`.
