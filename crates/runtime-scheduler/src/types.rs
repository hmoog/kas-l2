use std::{hash::Hash, sync::Arc};

use crate::resource_guard::ResourceGuard;

pub trait Element {
    type ResourceID: Eq + Hash + Clone;

    fn read_locks(&self) -> &[Self::ResourceID];

    fn write_locks(&self) -> &[Self::ResourceID];
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    ReadAccess = 0,
    WriteAccess = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardStatus {
    Waiting = 0,
    Ready = 1,
    Done = 2,
}

mod traits {
    use crate::types::{AccessType, GuardStatus};

    impl From<AccessType> for u8 {
        fn from(s: AccessType) -> Self {
            s as u8
        }
    }

    impl TryFrom<u8> for AccessType {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(AccessType::ReadAccess),
                1 => Ok(AccessType::WriteAccess),
                _ => Err(()),
            }
        }
    }

    impl From<GuardStatus> for u8 {
        fn from(s: GuardStatus) -> Self {
            s as u8
        }
    }

    impl TryFrom<u8> for GuardStatus {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(GuardStatus::Waiting),
                1 => Ok(GuardStatus::Ready),
                2 => Ok(GuardStatus::Done),
                _ => Err(()),
            }
        }
    }
}

pub type PrevGuards<E> = Vec<Option<Arc<ResourceGuard<E>>>>;

pub type NewGuards<E> = Vec<Arc<ResourceGuard<E>>>;

pub type Guards<E> = (PrevGuards<E>, NewGuards<E>);
