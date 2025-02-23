mod gui;
mod runner;
mod scene;
mod widgets;

use crate::runner::Runner;
use iced_winit::winit;
use winit::event_loop::EventLoop;

pub fn main() -> Result<(), winit::error::EventLoopError> {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new()?;

    let mut runner = Runner::Loading;
    event_loop.run_app(&mut runner)
}
