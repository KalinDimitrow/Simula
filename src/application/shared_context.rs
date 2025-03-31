use crate::application::CustomEventProxy;
use std::sync::RwLock;

#[derive(Debug)]
pub struct GeneralParams {
    pub lattice_dimension: (usize, usize),
    pub algorithm_started: bool,
}

pub(super) mod internal {
    use super::*;
    // #[derive(Debug)]
    #[derive(Debug)]
    pub struct SharedContext {
        pub event_proxy: CustomEventProxy,
        pub general_params: RwLock<GeneralParams>,
    }

    impl SharedContext {}
}

use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct SharedContext(Arc<internal::SharedContext>);

impl SharedContext {
    pub fn new(event_proxy: CustomEventProxy, dimensions: (usize, usize)) -> Self {
        SharedContext(Arc::new(internal::SharedContext {
            event_proxy,
            general_params: RwLock::new(GeneralParams {
                lattice_dimension: dimensions,
                algorithm_started: false,
            }),

        }))
    }

    pub fn lock(&self) -> &internal::SharedContext {
        self.0.as_ref()
    }
}
