use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global stack of prover randomness bytes.
pub static PROVER_RANDOMNESS: Lazy<Mutex<Vec<u8>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Prover/test harness API to preload deterministic randomness.
pub fn push_prover_randomness(data: &[u8]) {
    let mut buf = PROVER_RANDOMNESS.lock().unwrap();
    buf.extend_from_slice(data);
}
