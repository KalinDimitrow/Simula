use crate::algorithm_processor::{self, *};
pub(super) mod internal {
    use super::*;
    pub struct SharedContext {
        pub latice_dimentions: (usize, usize),
    }

    impl SharedContext {}
}

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SharedContext(Arc<Mutex<internal::SharedContext>>);

impl SharedContext {
    pub fn new(dimentions: (usize, usize)) -> Self {
        SharedContext(Arc::new(Mutex::new(internal::SharedContext {
            latice_dimentions: dimentions,
        })))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<internal::SharedContext> {
        self.0.lock().unwrap()
    }
}
