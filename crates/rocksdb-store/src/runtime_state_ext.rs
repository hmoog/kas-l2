use kas_l2_runtime_core::RuntimeState;
use rocksdb::{ColumnFamilyDescriptor, Options};

use crate::config::{Config, DefaultConfig};

pub trait RuntimeStateExt<C: Config = DefaultConfig> {
    fn cf_name(&self) -> &'static str;
    fn all_descriptors() -> Vec<ColumnFamilyDescriptor>;
}

impl<C: Config> RuntimeStateExt<C> for RuntimeState {
    fn cf_name(&self) -> &'static str {
        match self {
            RuntimeState::Data => "data",
            RuntimeState::DataPointers => "data_pointers",
            RuntimeState::Diffs => "diffs",
            RuntimeState::Metas => "metas",
        }
    }

    fn all_descriptors() -> Vec<ColumnFamilyDescriptor> {
        use RuntimeState::*;
        let cf_name = <RuntimeState as RuntimeStateExt<C>>::cf_name;
        vec![
            ColumnFamilyDescriptor::new("default", Options::default()),
            ColumnFamilyDescriptor::new(cf_name(&Data), C::cf_data_opts()),
            ColumnFamilyDescriptor::new(cf_name(&DataPointers), C::cf_latest_pointers_opts()),
            ColumnFamilyDescriptor::new(cf_name(&Diffs), C::cf_old_pointers_opts()),
            ColumnFamilyDescriptor::new(cf_name(&Metas), C::cf_meta_opts()),
        ]
    }
}
