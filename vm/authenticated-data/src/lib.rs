use kas_l2_vm_address::Address;
use kas_l2_vm_data::Data;

pub struct AuthenticatedData {
    address: Address,
    data: Data,
    mut_cap: Option<Data>,
}

impl AuthenticatedData {
    pub fn new(address: Address, data: Data, mut_cap: Option<Data>) -> Self {
        Self { address, data, mut_cap }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn mut_cap(&self) -> Option<&Data> {
        self.mut_cap.as_ref()
    }
}
