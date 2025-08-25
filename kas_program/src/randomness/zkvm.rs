use getrandom::Error;
use crate::randomness::buffer::PROVER_RANDOMNESS;

pub fn fill_bytes(buf: &mut [u8]) -> Result<(), Error> {
    let mut store = PROVER_RANDOMNESS.lock().unwrap();

    if store.len() < buf.len() {
        return Err(Error::from(Error::UNSUPPORTED));
    }

    let n = store.len();
    let tail = store.split_off(n - buf.len());
    buf.copy_from_slice(&tail);

    Ok(())
}
