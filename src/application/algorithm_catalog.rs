use crate::algorithm_processor::algorithm::Algorithm;
use crate::algorithm_processor::algorithm::*;

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

struct RandomRotation {}

impl AlgorithmComputation for RandomRotation {
    fn compute(&mut self, iteration: usize, data: &Vec<DataType>) -> Vec<DataType> {
        let (data, latice_dimentions) = match data[0] {
            DataType::OrientedGrid(ref data, latice_dimentions) => (data, latice_dimentions)
        };
        vec![DataType::OrientedGrid(data
                                        .iter()
                                        .map(|e| e * (iteration as f32) * std::f32::consts::PI / 180.0)
                                        .collect(), latice_dimentions)]
    }
}
