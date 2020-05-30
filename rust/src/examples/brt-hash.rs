use std::env;
use std::time::Instant;

use naturalize::to_natural;
use sha2::Sha256;

use brangetree::{hash_zipped, Error};

fn natural_sort<I: IntoIterator<Item = String>>(paths: I) -> Vec<String> {
    let mut paths: Vec<(String, String)> = paths
        .into_iter()
        .map(|arg| (to_natural(&arg).unwrap(), arg))
        .collect();
    paths.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    paths.into_iter().map(|a| a.1).collect()
}

fn main() -> Result<(), Error> {
    let paths: Vec<String> = natural_sort(env::args().skip(1));
    for path in paths {
        let start = Instant::now();
        let mut result = hash_zipped::<Sha256>(path.clone(), true)?;
        let dur = Instant::now() - start;

        if let Some(root) = result.root.take() {
            println!(
                "{} {} {} {} {:0.3}",
                path,
                result.filled_count,
                result.leaf_count,
                hex::encode(root),
                dur.as_secs_f64()
            );
        } else {
            println!("{} no hash produced", path);
        }
    }
    Ok(())
}
