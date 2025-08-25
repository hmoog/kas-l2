use getrandom::Error;

#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), Error> {
    let buf = unsafe { core::slice::from_raw_parts_mut(dest, len) };

    #[cfg(target_os = "zkvm")]
    {
        super::zkvm::fill_bytes(buf)
    }

    #[cfg(not(target_os = "zkvm"))]
    {
        super::host::fill_bytes(buf)
    }
}
