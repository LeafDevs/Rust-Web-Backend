use std::{env};
use argonautica::{Hasher, Verifier};

pub fn hash_password(password: &str) -> String {
    let hash_secret = env::var("HASH_SECRET").unwrap(); // Unwrap for simplicity (no error handling)
    
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(password)
        .with_secret_key(&hash_secret)
        // Recommended Argon2 parameters (adjust based on your security requirements)
        .configure_iterations(30)      // 3,000 iterations (more iterations = better security)
        .configure_memory_size(4096)    // 64 MB memory (higher is more secure, adjust based on hardware)
        .configure_lanes(8)               // 8 lanes (higher = better parallelization, use more CPU)
        .configure_hash_len(256)          // 256-byte hash (standard for high security)
        .configure_variant(argonautica::config::Variant::Argon2id)
        .hash()
        .unwrap(); // Unwrap here as well, assuming no error

    hash
}


pub fn verify_password(password: &str, hash: &str) -> rusqlite::Result<bool> {
    let hash_secret = env::var("HASH_SECRET").map_err(|_| rusqlite::Error::InvalidQuery)?;
    let mut verifier = Verifier::default();
    print!("{hash}");
    let is_valid = verifier
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(hash_secret)
        .verify()
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    Ok(is_valid)
}