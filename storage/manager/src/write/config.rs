use std::time::Duration;

#[derive(Clone, Debug)]
pub struct WriteConfig {
    max_batch_size: usize,
    max_batch_duration: Duration,
}

impl WriteConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_batch_size(mut self, max_batch_size: usize) -> Self {
        self.max_batch_size = max_batch_size;
        self
    }

    pub fn with_max_batch_duration(mut self, max_batch_duration: Duration) -> Self {
        self.max_batch_duration = max_batch_duration;
        self
    }

    pub fn max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    pub fn max_batch_duration(&self) -> Duration {
        self.max_batch_duration
    }
}

impl Default for WriteConfig {
    fn default() -> Self {
        Self { max_batch_size: 1000, max_batch_duration: Duration::from_millis(10) }
    }
}
