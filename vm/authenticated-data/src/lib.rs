use kas_l2_vm_data::Data;

pub struct AuthenticatedData {
    data: Data,
    mut_cap: Option<Data>,
}

impl AuthenticatedData {
    pub fn new(data: Data, mut_cap: Option<Data>) -> Self {
        Self { data, mut_cap }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn mut_cap(&self) -> Option<&Data> {
        self.mut_cap.as_ref()
    }
}
