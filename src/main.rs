mod algorithm_processor;
mod application;
mod gui;
mod rendering;
mod widgets;

use crate::application::Simula;
use iced_winit::winit;
use winit::event_loop::EventLoop;

pub fn main() -> Result<(), winit::error::EventLoopError> {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new()?;
    let proxy = event_loop.create_proxy();
    let mut app = Simula::new(proxy.clone());
    event_loop.run_app(&mut app)
}
