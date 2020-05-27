use std::env;
use std::time::Instant;

use brangetree::{find_merkle_path, Digest, Error, PathJoin, Sha256};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().skip(1).take(2).collect();
    if args.len() != 2 {
        println!("Expected two arguments: path and index");
    } else {
        let index = args[1].parse::<u32>().unwrap();
        let start = Instant::now();
        let (found_range, found_path, mut result) =
            find_merkle_path::<Sha256>(args[0].clone(), index)?;
        let dur = Instant::now() - start;

        if let Some(root) = result.root.take() {
            if let (Some(found_range), Some(found_path)) = (found_range, found_path) {
                let verify = found_path.clone().fold(|l, r| {
                    let mut hash = Sha256::new();
                    hash.input(l);
                    hash.input(r);
                    hash.result().to_vec()
                });

                let mut path_parts = vec![hex::encode(found_path.leaf)];
                for part in found_path.join {
                    let (pfx, hash) = match part {
                        PathJoin::Left(h) => ("L", h),
                        PathJoin::Right(h) => ("R", h),
                    };
                    path_parts.push(format!("{} {}", pfx, hex::encode(hash)));
                }

                println!("found range: {:?}", found_range);
                println!("hash chain:  {:?}", path_parts);
                println!("verify hash: {}", hex::encode(verify));
            } else {
                println!("index not found in non-revoked range");
            }
            println!("root hash    {}", hex::encode(root));
            println!("leaf count:  {}", result.leaf_count);
            println!("duration:    {:0.3}", dur.as_secs_f64());
        } else {
            println!("{} no hash produced", args[0]);
        }
    }
    Ok(())
}
