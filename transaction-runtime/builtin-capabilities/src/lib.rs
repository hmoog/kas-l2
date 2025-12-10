use kas_l2_transaction_runtime_address::Address;
use kas_l2_transaction_runtime_data::Data;

pub struct AccessGranted;

impl From<AccessGranted> for Data {
    fn from(_: AccessGranted) -> Self {
        Data::new(Address::SYSTEM, 0, vec![])
    }
}
