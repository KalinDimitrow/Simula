mod shared_context;
mod wininit_wrapper;
mod components;

pub use shared_context::*;
use crate::gui::controls::Controls;
use crate::rendering::*;
use winit::event_loop::EventLoopProxy;
use self::wininit_wrapper::WininitWrapper;
use self::components::Components;

use winit::event::WindowEvent;
use crate::rendering::wgpu_wrapper::WGPUWrapper;

#[derive(Debug)]
pub enum CustomEvent {
    RequestRedraw,
}

pub type CustomEventProxy = EventLoopProxy<CustomEvent>;

#[allow(clippy::large_enum_variant)]
pub enum Simula {
    Loading(CustomEventProxy),
    Ready {
        components: Components,
    },
}

impl Simula {
    pub fn new(event_proxy: CustomEventProxy) -> Self {
        Self::Loading(event_proxy)
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
        win: &mut WininitWrapper,
        wgpu: &mut WGPUWrapper,
        state: &program::State<Controls>,
        debug: &Debug,
    ) {
        if win.resized {
            let size = win.window.inner_size();

            win.viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                win.window.scale_factor(),
            );

            wgpu.surface.configure(
                &wgpu.device,
                &SurfaceConfiguration {
                    format: wgpu.format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    width: size.width,
                    height: size.height,
                    present_mode: PresentMode::AutoVsync,
                    alpha_mode: CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            win.resized = false;
        }

        match wgpu.surface.get_current_texture() {
            Ok(frame) => {
                Simula::render(
                    frame, &wgpu.device, state, &win.viewport, &mut wgpu.renderer, &mut wgpu.engine, &wgpu.queue, &win.window, &debug,
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
                    win.window.request_redraw();
                }
            },
        }
    }
}

impl winit::application::ApplicationHandler<CustomEvent> for Simula {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Loading(event_proxy) = self {
            *self =  Self::Ready {components: Components::new(event_proxy.clone(), event_loop)}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Ready {
            components
        } = self
        else {
            return;
        };

        components.background_renderer.render(&mut components.wgpu);

        match event {
            WindowEvent::RedrawRequested => {
                Simula::handle_redraw_event(
                    &mut components.win,
                    &mut components.wgpu,
                    &components.state,
                    &components.debug,
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                components.win.cursor_position = Some(position);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                components.win.modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(_) => {
                components.win.resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) =
            conversion::window_event(event, components.win.window.scale_factor(), components.win.modifiers)
        {
            components.state.queue_event(event);
        }

        // If there are events pending
        if !components.state.is_queue_empty() {
            // We update iced
            let _ = components.state.update(
                components.win.viewport.logical_size(),
                components.win.cursor_position
                    .map(|p| conversion::cursor_position(p, components.win.viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                &mut components.wgpu.renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                &mut components.win.clipboard,
                &mut components.debug,
            );
            // and request a redraw
            components.win.window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: CustomEvent) {
        #[allow(unused_variables)]
        let Self::Ready {
            components
        } = self
        else {
            return;
        };

        match event {
            CustomEvent::RequestRedraw => {
                components.win.window.request_redraw();
            }
        }
    }
}
