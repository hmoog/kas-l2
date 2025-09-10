#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStatus {
    Requested = 0,
    Acquired = 1,
    Released = 2,
}

impl From<LockStatus> for u8 {
    fn from(s: LockStatus) -> Self {
        s as u8
    }
}

impl TryFrom<u8> for LockStatus {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(LockStatus::Requested),
            1 => Ok(LockStatus::Acquired),
            2 => Ok(LockStatus::Released),
            _ => Err(()),
        }
    }
}
