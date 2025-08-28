#[derive(Clone, Debug)]
pub struct Account {
    pub key: [u8; 32],
    pub owner: [u8; 32],
    pub lamports: u64,
    pub data: Vec<u8>,
    pub is_signer: bool,
    pub is_writable: bool,
    pub executable: bool,
    pub rent_epoch: u64,
    /// If Some(j), this entry is a *duplicate* of account j (0-based).
    /// If None, a full account record is serialized.
    pub dup_of: Option<usize>,
}
