use std::boxed;

pub struct Algorithm {
    processor: Box<dyn AlgorithmProcessor>,
    renderer: Box<dyn AlgorithmRenderer>,
    gui: Box<dyn AlgorithmGUI>,
}

pub trait AlgorithmProcessor {
    fn process(&mut self);
}

pub trait AlgorithmRenderer {
    fn render(&mut self);
}

pub trait AlgorithmGUI {
    fn gui(&mut self);
}
