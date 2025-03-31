use crate::algorithm_processor::algorithm::Algorithm;

pub struct AlgorithmCatalog {
    pub algorithms: Vec<Algorithm>,
}

impl AlgorithmCatalog {
    pub fn new() -> Self {
        Self {
            algorithms: Vec::new(),
        }
    }
}
