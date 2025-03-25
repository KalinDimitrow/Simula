use crate::rendering::winit::event_loop::ControlFlow;
use crate::rendering::winit::keyboard::ModifiersState;
use crate::rendering::{Size, Viewport, winit};
use iced_winit::Clipboard;
use std::sync::Arc;

pub struct WininitWrapper {
    pub window: Arc<winit::window::Window>,
    pub cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    pub clipboard: Clipboard,
    pub viewport: Viewport,
    pub modifiers: ModifiersState,
    pub resized: bool,
}

impl WininitWrapper {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let window = Self::create_window(event_loop);
        let viewport = Self::get_viewport(&*window);
        let clipboard = Clipboard::connect(window.clone());
        event_loop.set_control_flow(ControlFlow::Wait);
        Self {
            window,
            cursor_position: None,
            clipboard,
            viewport,
            modifiers: ModifiersState::default(),
            resized: false,
        }
    }

    fn create_window(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Arc<winit::window::Window> {
        let mut window_attributes = winit::window::WindowAttributes::default();
        window_attributes.title = "Simula".to_owned();
        window_attributes.maximized = true;
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Create window"),
        );

        window
    }

    fn get_viewport(window: &winit::window::Window) -> Viewport {
        let physical_size = window.inner_size();
        Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        )
    }
}
