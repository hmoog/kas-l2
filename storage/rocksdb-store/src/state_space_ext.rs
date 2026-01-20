use rocksdb::ColumnFamilyDescriptor;
use vprogs_state_space::StateSpace;

use crate::config::{Config, DefaultConfig};

pub trait StateSpaceExt<C: Config = DefaultConfig> {
    fn cf_name(&self) -> &'static str;
    fn all_descriptors() -> Vec<ColumnFamilyDescriptor>;
}

impl<C: Config> StateSpaceExt<C> for StateSpace {
    fn cf_name(&self) -> &'static str {
        match self {
            StateSpace::Data => "data",
            StateSpace::LatestPtr => "latest_ptr",
            StateSpace::RollbackPtr => "rollback_ptr",
            StateSpace::Metas => "metas",
        }
    }

    fn all_descriptors() -> Vec<ColumnFamilyDescriptor> {
        use StateSpace::*;
        let cf_name = <StateSpace as StateSpaceExt<C>>::cf_name;
        vec![
            ColumnFamilyDescriptor::new(cf_name(&Data), C::cf_data_opts()),
            ColumnFamilyDescriptor::new(cf_name(&LatestPtr), C::cf_latest_ptr_opts()),
            ColumnFamilyDescriptor::new(cf_name(&RollbackPtr), C::cf_rollback_ptr_opts()),
            ColumnFamilyDescriptor::new(cf_name(&Metas), C::cf_metas_opts()),
        ]
    }
}
