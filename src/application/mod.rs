mod shared_context;
pub use shared_context::*;

use crate::algorithm_processor::{self, *};
use crate::gui::controls::Controls;
use crate::rendering::renderers::*;
use crate::rendering::*;
use winit::event_loop::EventLoopProxy;

use std::sync::Arc;

use winit::{event::WindowEvent, event_loop::ControlFlow, keyboard::ModifiersState};
use crate::rendering::webgpu_wrapper::WebGPUWrapper;

#[derive(Debug)]
pub enum CustomEvent {
    RequestRedraw,
}

pub type CustomEventProxy = EventLoopProxy<CustomEvent>;

#[allow(clippy::large_enum_variant)]
enum DrawingContext {
    Loading,
    Ready {
        window: Arc<winit::window::Window>,
        web_gpuwrapper: WebGPUWrapper,
        cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
        clipboard: Clipboard,
        viewport: Viewport,
        modifiers: ModifiersState,
        resized: bool,
        state: program::State<Controls>,
        algorithm_processor: AlgorithmProcessor,
        background_renderer: BackgroundRenderer,
    },
}
pub struct Simula {
    drawing_context: DrawingContext,
    shared_context: SharedContext,
    event_proxy: CustomEventProxy,
    debug: Debug,
}

impl Simula {
    pub fn new(event_proxy: CustomEventProxy) -> Self {
        Self {
            drawing_context: DrawingContext::Loading,
            shared_context: SharedContext::new((300, 300)),
            event_proxy,
            debug: Debug::new(),
        }
    }

    fn render(
        frame: SurfaceTexture,
        device: &Device,
        state: &program::State<Controls>,
        viewport: &Viewport,
        renderer: &mut Renderer,
        engine: &mut Engine,
        queue: &Queue,
        window: &winit::window::Window,
        debug: &Debug,
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        renderer.present(
            engine,
            device,
            queue,
            &mut encoder,
            None,
            frame.texture.format(),
            &view,
            viewport,
            &debug.overlay(),
        );

        // Then we submit the work
        engine.submit(queue, encoder);
        frame.present();

        // Update the mouse cursor
        window.set_cursor(iced_winit::conversion::mouse_interaction(
            state.mouse_interaction(),
        ));
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
    
    fn handle_redraw_event(
        resized: &mut bool,
        window: &winit::window::Window,
        viewport: &mut Viewport,
        web_gpuwrapper: &mut WebGPUWrapper,
        state: &program::State<Controls>,
        debug: &Debug,
    ) {
        if *resized {
            let size = window.inner_size();

            *viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                window.scale_factor(),
            );

            web_gpuwrapper.surface.configure(
                &web_gpuwrapper.device,
                &SurfaceConfiguration {
                    format: web_gpuwrapper.format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    width: size.width,
                    height: size.height,
                    present_mode: PresentMode::AutoVsync,
                    alpha_mode: CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            *resized = false;
        }

        match web_gpuwrapper.surface.get_current_texture() {
            Ok(frame) => {
                Simula::render(
                    frame, &web_gpuwrapper.device, state, viewport, &mut web_gpuwrapper.renderer, &mut web_gpuwrapper.engine, &web_gpuwrapper.queue, window, &debug,
                );
            }
            Err(error) => match error {
                SurfaceError::OutOfMemory => {
                    panic!(
                        "Swapchain error: {error}. \
                    Rendering cannot continue."
                    )
                }
                _ => {
                    // Try rendering again next frame.
                    window.request_redraw();
                }
            },
        }
    }
}

impl winit::application::ApplicationHandler<CustomEvent> for Simula {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let DrawingContext::Loading = self.drawing_context {
            let window = Self::create_window(event_loop);
            let viewport = Self::get_viewport(&*window);
            let clipboard = Clipboard::connect(window.clone());
            let mut web_gpuwrapper = WebGPUWrapper::new(window.clone());

            let (data_handle, mut algorithm_processor) =
                algorithm_processor::AlgorithmProcessor::new(self.event_proxy.clone());
            algorithm_processor.start(self.shared_context.clone());
            let background_renderer = BackgroundRenderer::new(
                &web_gpuwrapper,
                &viewport,
                data_handle,
                self.shared_context.clone(),
            );
            algorithm_processor.start(self.shared_context.clone());
            let state = program::State::new(
                Controls::new(background_renderer.get_texture_handle()),
                viewport.logical_size(),
                &mut web_gpuwrapper.renderer,
                &mut self.debug,
            );

            event_loop.set_control_flow(ControlFlow::Wait);

            self.drawing_context = DrawingContext::Ready {
                window,
                web_gpuwrapper,
                cursor_position: None,
                modifiers: ModifiersState::default(),
                clipboard,
                viewport,
                resized: true,
                state,
                algorithm_processor,
                background_renderer,
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        #[allow(unused_variables)]
        let DrawingContext::Ready {
            window,
            web_gpuwrapper,
            viewport,
            cursor_position,
            modifiers,
            clipboard,
            resized,
            state,
            algorithm_processor,
            background_renderer,
        } = &mut self.drawing_context
        else {
            return;
        };

        background_renderer.render(web_gpuwrapper);

        match event {
            WindowEvent::RedrawRequested => {
                Simula::handle_redraw_event(
                    resized,
                    window,
                    viewport,
                    web_gpuwrapper,
                    state,
                    &self.debug,
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                *cursor_position = Some(position);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                *modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(_) => {
                *resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) =
            iced_winit::conversion::window_event(event, window.scale_factor(), *modifiers)
        {
            state.queue_event(event);
        }

        // If there are events pending
        if !state.is_queue_empty() {
            // We update iced
            let _ = state.update(
                viewport.logical_size(),
                cursor_position
                    .map(|p| conversion::cursor_position(p, viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                &mut web_gpuwrapper.renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                clipboard,
                &mut self.debug,
            );
            // and request a redraw
            window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: CustomEvent) {
        #[allow(unused_variables)]
        let DrawingContext::Ready {
            window,
            web_gpuwrapper,
            viewport,
            cursor_position,
            modifiers,
            clipboard,
            resized,
            state,
            algorithm_processor,
            background_renderer,
        } = &mut self.drawing_context
        else {
            return;
        };

        match event {
            CustomEvent::RequestRedraw => {
                window.request_redraw();
            }
        }
    }
}
