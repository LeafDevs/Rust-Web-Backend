use std::{env};
use argonautica::{Hasher, Verifier};
use rusqlite::{params, Connection};

pub fn hash_password(password: &str) -> String {
    let hash_secret = env::var("HASH_SECRET").unwrap(); // Unwrap for simplicity (no error handling)
    
    let mut hasher = Hasher::default();
    hasher
        .with_password(password)
        .with_secret_key(&hash_secret)
        // Recommended Argon2 parameters (adjust based on your security requirements)
        .configure_iterations(1_000)      // 3,000 iterations (more iterations = better security)
        .configure_memory_size(131072 / 2)    // 64 MB memory (higher is more secure, adjust based on hardware)
        .configure_lanes(8)               // 8 lanes (higher = better parallelization, use more CPU)
        .configure_hash_len(256)          // 256-byte hash (standard for high security)
        .configure_variant(argonautica::config::Variant::Argon2id);
    

    let hash = hasher.hash().unwrap(); // Unwrap here as well, assuming no error

    hash
}


// pub fn verify_password(username: &str, password: &str) -> Result<bool> {
//     // Fetch stored hash from database
//     let conn = Connection::open("fbla.db")?;
//     let mut stmt = conn.prepare(
//         "SELECT password_hash FROM users WHERE username = ?1",
//     )?;
    
//     let stored_hash: String = stmt.query_row(params![username], |row| row.get(0))?;

//     // Get secret key from environment
//     let hash_secret = env::var("HASH_SECRET")?;

//     // Verify password against stored hash
//     Verifier::default()
//         .with_hash(stored_hash)
//         .with_password(password)
//         .with_secret_key(hash_secret)
//         .verify()
//         .map_err(|e| e.into())
// }