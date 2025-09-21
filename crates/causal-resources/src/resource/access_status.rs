#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStatus {
    Waiting = 0,
    Ready = 1,
    Done = 2,
}

impl From<AccessStatus> for u8 {
    fn from(s: AccessStatus) -> Self {
        s as u8
    }
}

impl TryFrom<u8> for AccessStatus {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(AccessStatus::Waiting),
            1 => Ok(AccessStatus::Ready),
            2 => Ok(AccessStatus::Done),
            _ => Err(()),
        }
    }
}
