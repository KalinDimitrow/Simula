mod shared_context;
mod wininit_wrapper;

pub use shared_context::*;

use crate::algorithm_processor::{self, *};
use crate::gui::controls::Controls;
use crate::rendering::renderers::*;
use crate::rendering::*;
use winit::event_loop::EventLoopProxy;
use self::wininit_wrapper::WininitWrapper;

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
        wininit_wrapper: WininitWrapper,
        web_gpuwrapper: WebGPUWrapper,
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
        window.set_cursor(conversion::mouse_interaction(
            state.mouse_interaction(),
        ));
    }

    fn handle_redraw_event(
        wininit_wrapper: &mut WininitWrapper,
        web_gpuwrapper: &mut WebGPUWrapper,
        state: &program::State<Controls>,
        debug: &Debug,
    ) {
        if wininit_wrapper.resized {
            let size = wininit_wrapper.window.inner_size();

            wininit_wrapper.viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                wininit_wrapper.window.scale_factor(),
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

            wininit_wrapper.resized = false;
        }

        match web_gpuwrapper.surface.get_current_texture() {
            Ok(frame) => {
                Simula::render(
                    frame, &web_gpuwrapper.device, state, &wininit_wrapper.viewport, &mut web_gpuwrapper.renderer, &mut web_gpuwrapper.engine, &web_gpuwrapper.queue, &wininit_wrapper.window, &debug,
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
                    wininit_wrapper.window.request_redraw();
                }
            },
        }
    }
}

impl winit::application::ApplicationHandler<CustomEvent> for Simula {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let DrawingContext::Loading = self.drawing_context {
            let wininit_wrapper = WininitWrapper::new(event_loop);
            let mut web_gpuwrapper = WebGPUWrapper::new(wininit_wrapper.window.clone());

            let (data_handle, mut algorithm_processor) =
                algorithm_processor::AlgorithmProcessor::new(self.event_proxy.clone());
            algorithm_processor.start(self.shared_context.clone());
            let background_renderer = BackgroundRenderer::new(
                &web_gpuwrapper,
                &wininit_wrapper.viewport,
                data_handle,
                self.shared_context.clone(),
            );
            algorithm_processor.start(self.shared_context.clone());
            let state = program::State::new(
                Controls::new(background_renderer.get_texture_handle()),
                wininit_wrapper.viewport.logical_size(),
                &mut web_gpuwrapper.renderer,
                &mut self.debug,
            );

            event_loop.set_control_flow(ControlFlow::Wait);

            self.drawing_context = DrawingContext::Ready {
                wininit_wrapper,
                web_gpuwrapper,
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
            wininit_wrapper,
            web_gpuwrapper,
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
                    wininit_wrapper,
                    web_gpuwrapper,
                    state,
                    &self.debug,
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                wininit_wrapper.cursor_position = Some(position);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                wininit_wrapper.modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(_) => {
                wininit_wrapper.resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) =
            conversion::window_event(event, wininit_wrapper.window.scale_factor(), wininit_wrapper.modifiers)
        {
            state.queue_event(event);
        }

        // If there are events pending
        if !state.is_queue_empty() {
            // We update iced
            let _ = state.update(
                wininit_wrapper.viewport.logical_size(),
                wininit_wrapper.cursor_position
                    .map(|p| conversion::cursor_position(p, wininit_wrapper.viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                &mut web_gpuwrapper.renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                &mut wininit_wrapper.clipboard,
                &mut self.debug,
            );
            // and request a redraw
            wininit_wrapper.window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: CustomEvent) {
        #[allow(unused_variables)]
        let DrawingContext::Ready {
            wininit_wrapper,
            web_gpuwrapper,
            state,
            algorithm_processor,
            background_renderer,
        } = &mut self.drawing_context
        else {
            return;
        };

        match event {
            CustomEvent::RequestRedraw => {
                wininit_wrapper.window.request_redraw();
            }
        }
    }
}
