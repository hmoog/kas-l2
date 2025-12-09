use kas_l2_object_auth_capabilities::ObjectLock;

pub struct ThresholdLock {
    pub child_locks: Vec<ObjectLock>,
    pub threshold: u64,
}

impl ThresholdLock {
    pub fn new(child_locks: Vec<ObjectLock>, threshold: u64) -> Self {
        Self { child_locks, threshold }
    }
}
