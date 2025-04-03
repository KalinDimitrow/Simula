use std::boxed;

pub enum DataType {
    OrientedGrid(Vec<f32>, (usize, usize)),
}

pub struct Algorithm {
    processor: Box<dyn AlgorithmComputation>,
    renderer: Box<dyn AlgorithmRenderer>,
    gui: Box<dyn AlgorithmGUI>,
}

pub trait AlgorithmComputation {
    fn compute(&mut self, iteration: usize, data: &Vec<DataType>) -> Vec<DataType>;
}

pub trait AlgorithmRenderer {
    fn render(&mut self);
}

pub trait AlgorithmGUI {
    fn gui(&mut self);
}
