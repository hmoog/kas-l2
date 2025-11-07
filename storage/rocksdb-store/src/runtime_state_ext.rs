use kas_l2_runtime::RuntimeState;
use rocksdb::ColumnFamilyDescriptor;

use crate::config::{Config, DefaultConfig};

pub trait RuntimeStateExt<C: Config = DefaultConfig> {
    fn cf_name(&self) -> &'static str;
    fn all_descriptors() -> Vec<ColumnFamilyDescriptor>;
}

impl<C: Config> RuntimeStateExt<C> for RuntimeState {
    fn cf_name(&self) -> &'static str {
        match self {
            RuntimeState::Data => "data",
            RuntimeState::LatestPtr => "latest_ptr",
            RuntimeState::RollbackPtr => "rollback_ptr",
            RuntimeState::Metas => "metas",
        }
    }

    fn all_descriptors() -> Vec<ColumnFamilyDescriptor> {
        use RuntimeState::*;
        let cf_name = <RuntimeState as RuntimeStateExt<C>>::cf_name;
        vec![
            ColumnFamilyDescriptor::new(cf_name(&Data), C::cf_data_opts()),
            ColumnFamilyDescriptor::new(cf_name(&LatestPtr), C::cf_latest_ptr_opts()),
            ColumnFamilyDescriptor::new(cf_name(&RollbackPtr), C::cf_rollback_ptr_opts()),
            ColumnFamilyDescriptor::new(cf_name(&Metas), C::cf_metas_opts()),
        ]
    }
}
