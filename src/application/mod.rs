mod shared_context;
mod wininit_wrapper;
mod components;

pub use shared_context::*;
use crate::rendering::*;
use winit::event_loop::EventLoopProxy;
use self::components::Components;

use winit::event::WindowEvent;

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
        components: &mut Components
    ) {
        let mut encoder = components.wgpu.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        components.wgpu.renderer.present(
            &mut components.wgpu.engine,
            &mut components.wgpu.device,
            &mut components.wgpu.queue,
            &mut encoder,
            None,
            frame.texture.format(),
            &view,
            &components.win.viewport,
            &components.debug.overlay(),
        );

        // Then we submit the work
        components.wgpu.engine.submit(&components.wgpu.queue, encoder);
        frame.present();

        // Update the mouse cursor
        components.win.window.set_cursor(conversion::mouse_interaction(
            components.state.mouse_interaction(),
        ));
    }

    fn handle_redraw_event(
        components: &mut Components
    ) {
        if components.win.resized {
            let size = components.win.window.inner_size();

            components.win.viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                components.win.window.scale_factor(),
            );

            components.wgpu.surface.configure(
                &components.wgpu.device,
                &SurfaceConfiguration {
                    format: components.wgpu.format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    width: size.width,
                    height: size.height,
                    present_mode: PresentMode::AutoVsync,
                    alpha_mode: CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            components.win.resized = false;
        }

        match components.wgpu.surface.get_current_texture() {
            Ok(frame) => {
                Simula::render(
                    frame, components,
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
                    components.win.window.request_redraw();
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
                Simula::handle_redraw_event(components);
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
