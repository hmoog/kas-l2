use std::sync::Arc;

use crate::{ResourceID, access_metadata::AccessMetadata, resource_state::ResourceState};

pub enum ResourceHandle<ID: ResourceID, A: AccessMetadata<ID>> {
    Read {
        state: Arc<ResourceState<ID>>, // shared snapshot
        access_metadata: A,
    },
    Write {
        state: ResourceState<ID>, // owned scratchpad
        access_metadata: A,
    },
}
