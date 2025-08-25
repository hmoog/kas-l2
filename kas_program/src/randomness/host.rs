use getrandom::Error;

#[cfg(feature = "std")]
pub fn fill_bytes(buf: &mut [u8]) -> Result<(), Error> {
    use std::io::Read;
    let mut f = std::fs::File::open("/dev/urandom").map_err(|_| Error::UNSUPPORTED)?;
    f.read_exact(buf).map_err(|_| Error::UNSUPPORTED)?;
    Ok(())
}

#[cfg(not(feature = "std"))]
pub fn fill_bytes(_buf: &mut [u8]) -> Result<(), Error> {
    Err(Error::from(Error::UNSUPPORTED))
}