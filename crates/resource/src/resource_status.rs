#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatus {
    Waiting = 0,
    Ready = 1,
    Done = 2,
}

impl From<ResourceStatus> for u8 {
    fn from(s: ResourceStatus) -> Self {
        s as u8
    }
}

impl TryFrom<u8> for ResourceStatus {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(ResourceStatus::Waiting),
            1 => Ok(ResourceStatus::Ready),
            2 => Ok(ResourceStatus::Done),
            _ => Err(()),
        }
    }
}
