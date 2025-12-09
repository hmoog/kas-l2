use std::{any::Any, fmt::Debug};

use crate::PubkeyType;

pub trait Capability<S: PubkeyType>: Debug + Sized + Any + Send + 'static {}
