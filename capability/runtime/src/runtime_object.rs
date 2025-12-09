use crate::{Capability, PubkeyType};
use crate::erased_capability::ErasedCapability;

pub struct RuntimeObject<P: PubkeyType, C: Capability<P>> {
    cap: ErasedCapability,
}