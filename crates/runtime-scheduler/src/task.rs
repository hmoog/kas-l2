use crate::task_id::TaskID;

pub trait Task {
    type ResourceID: TaskID;

    fn execute(&self);

    fn read_locks(&self) -> Vec<Self::ResourceID>;

    fn write_locks(&self) -> Vec<Self::ResourceID>;
}
