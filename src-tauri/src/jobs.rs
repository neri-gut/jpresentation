/// Background work pool. Empty in change 001; later changes enqueue index, thumbnail, and download jobs.
///
/// Size is `max(2, cores - 1)` so a modest hall PC keeps a core for the UI thread.
pub struct JobQueue {
    #[allow(dead_code)]
    pool_size: usize,
}

impl JobQueue {
    /// Builds an idle queue with a hardware-derived concurrency cap.
    pub fn new() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        let pool_size = std::cmp::max(2, cores.saturating_sub(1));
        Self { pool_size }
    }

    /// Worker slots reserved for later jobs. Never zero.
    #[allow(dead_code)]
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_size_is_at_least_two() {
        let queue = JobQueue::new();
        assert!(queue.pool_size() >= 2);
    }
}
