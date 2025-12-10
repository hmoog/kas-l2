use kas_l2_transaction_runtime_address::Address;

pub enum ProgramArg {
    StoredData(Address),
    ReturnedData(usize, usize),
    // TODO: Replace Vec<u8> with a more specific type representing scalar values
    Scalar(Vec<u8>),
}
