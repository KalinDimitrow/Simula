use crate::application::CustomEventProxy;

pub(super) mod internal {

    use super::*;
    #[derive(Debug)]
    pub struct SharedContext {
        pub event_proxy: CustomEventProxy,
        pub lattice_dimension: (usize, usize),
        pub algorithm_started: bool,
    }

    impl SharedContext {}
}

use std::sync::{Arc, Mutex};

#[derive(Clone,Debug)]
pub struct SharedContext(Arc<Mutex<internal::SharedContext>>);

impl SharedContext {
    pub fn new(event_proxy: CustomEventProxy, dimensions: (usize, usize)) -> Self {
        SharedContext(Arc::new(Mutex::new(internal::SharedContext {
            event_proxy,
            lattice_dimension: dimensions,
            algorithm_started: false,
        })))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<internal::SharedContext> {
        self.0.lock().unwrap()
    }
}
