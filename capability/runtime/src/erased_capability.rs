use std::any::Any;

pub type ErasedCapability = Box<dyn Any + Send>;