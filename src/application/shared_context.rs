pub(super) mod internal {
    use super::*;
    pub struct SharedContext {
        pub lattice_dimension: (usize, usize),
    }

    impl SharedContext {}
}

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SharedContext(Arc<Mutex<internal::SharedContext>>);

impl SharedContext {
    pub fn new(dimensions: (usize, usize)) -> Self {
        SharedContext(Arc::new(Mutex::new(internal::SharedContext {
            lattice_dimension: dimensions,
        })))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<internal::SharedContext> {
        self.0.lock().unwrap()
    }
}
