use std::sync::Arc;

use kas_l2_atomic::{AtomicEnum, AtomicWeak};

use crate::{scheduled_task::ScheduledTask, task::Task};

pub struct Guard<T: Task> {
    pub status: AtomicEnum<Status>,
    pub guard_type: AtomicEnum<Type>,
    pub successor: AtomicWeak<Guard<T>>,
    pub owner: AtomicWeak<ScheduledTask<T>>,
    pub owner_index: usize,
}

impl<T: Task> Guard<T> {
    pub fn new(access_type: Type, element_index: usize) -> Self {
        Self {
            guard_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(Status::Waiting),
            successor: AtomicWeak::new(),
            owner: AtomicWeak::new(),
            owner_index: element_index,
        }
    }

    pub fn extend(&self, successor: &Arc<Guard<T>>) {
        self.successor.store(Arc::downgrade(successor));

        match self.status.load() {
            Status::Ready if self.guard_type.load() == Type::ReadGuard => {
                if successor.guard_type.load() == Type::ReadGuard {
                    successor.ready();
                }
            }
            Status::Done => successor.ready(),
            _ => {} // do nothing, the successor will be notified when anything changes
        }
    }

    pub fn ready(&self) {
        if self.status.compare_exchange(Status::Waiting, Status::Ready).is_ok() {
            if let Some(owner) = self.owner.load().upgrade() {
                owner.notify_ready();
            }

            if self.guard_type.load() == Type::ReadGuard {
                if let Some(successor) = self.successor.load().upgrade() {
                    if successor.guard_type.load() == Type::ReadGuard {
                        successor.ready();
                    }
                }
            }
        }
    }

    pub fn done(&self) {
        if self.status.compare_exchange(Status::Ready, Status::Done).is_ok() {
            if let Some(successor) = self.successor.load().upgrade() {
                successor.ready();
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    ReadGuard = 0,
    WriteGuard = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Waiting = 0,
    Ready = 1,
    Done = 2,
}

pub type PrevGuards<E> = Vec<Option<Arc<Guard<E>>>>;

pub type NewGuards<E> = Vec<Arc<Guard<E>>>;

pub type Guards<E> = (PrevGuards<E>, NewGuards<E>);

mod traits {
    use super::{Status, Type};

    impl From<Type> for u8 {
        fn from(s: Type) -> Self {
            s as u8
        }
    }

    impl TryFrom<u8> for Type {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Type::ReadGuard),
                1 => Ok(Type::WriteGuard),
                _ => Err(()),
            }
        }
    }

    impl From<Status> for u8 {
        fn from(s: Status) -> Self {
            s as u8
        }
    }

    impl TryFrom<u8> for Status {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Status::Waiting),
                1 => Ok(Status::Ready),
                2 => Ok(Status::Done),
                _ => Err(()),
            }
        }
    }
}
