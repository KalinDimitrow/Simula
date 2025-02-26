use crate::gui::controls::Controls;
use crate::rendering::liquid_crystal_latice::scene::Scene;

use iced_wgpu::graphics::Viewport;
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::conversion;
use iced_winit::core::mouse;
use iced_winit::core::renderer;
use iced_winit::core::{Color, Font, Pixels, Size, Theme};
use iced_winit::futures;
use iced_winit::runtime::program;
use iced_winit::runtime::Debug;
use iced_winit::winit;
use iced_winit::Clipboard;
use std::sync::{Arc, Mutex};

use winit::{event::WindowEvent, event_loop::ControlFlow, keyboard::ModifiersState};

#[allow(clippy::large_enum_variant)]
pub enum Runner {
    Loading,
    Ready {
        window: Arc<winit::window::Window>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        format: wgpu::TextureFormat,
        engine: Engine,
        renderer: Renderer,
        scene: Scene,
        state: program::State<Controls>,
        cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
        clipboard: Clipboard,
        viewport: Viewport,
        modifiers: ModifiersState,
        resized: bool,
        debug: Debug,
        tex: Arc<Mutex<Option<iced_wgpu::wgpu::Texture>>>,
    },
}

impl winit::application::ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Loading = self {
            let window = Arc::new(
                event_loop
                    .create_window(winit::window::WindowAttributes::default())
                    .expect("Create window"),
            );

            let physical_size = window.inner_size();
            let viewport = Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor(),
            );
            let clipboard = Clipboard::connect(window.clone());

            let backend = wgpu::util::backend_bits_from_env().unwrap_or_default();

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: backend,
                ..Default::default()
            });
            let surface = instance
                .create_surface(window.clone())
                .expect("Create window surface");

            let (format, adapter, device, queue) = futures::futures::executor::block_on(async {
                let adapter =
                    wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                        .await
                        .expect("Create adapter");

                let adapter_features = adapter.features();

                let capabilities = surface.get_capabilities(&adapter);

                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: adapter_features & wgpu::Features::default(),
                            required_limits: wgpu::Limits::default(),
                        },
                        None,
                    )
                    .await
                    .expect("Request device");

                (
                    capabilities
                        .formats
                        .iter()
                        .copied()
                        .find(wgpu::TextureFormat::is_srgb)
                        .or_else(|| capabilities.formats.first().copied())
                        .expect("Get preferred format"),
                    adapter,
                    device,
                    queue,
                )
            });

            surface.configure(
                &device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width: physical_size.width,
                    height: physical_size.height,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            // Initialize scene and GUI controls
            let scene = Scene::new(&device, &queue, format);
            let tex = Arc::new(Mutex::new(None));
            let controls = Controls::new(tex.clone());

            // Initialize iced
            let mut debug = Debug::new();
            let engine = Engine::new(&adapter, &device, &queue, format, None);
            let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

            let state =
                program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

            // You should change this if you want to render continuously
            event_loop.set_control_flow(ControlFlow::Wait);

            *self = Self::Ready {
                window,
                device,
                queue,
                surface,
                format,
                engine,
                renderer,
                scene,
                state,
                cursor_position: None,
                modifiers: ModifiersState::default(),
                clipboard,
                viewport,
                resized: false,
                debug,
                tex,
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Ready {
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            scene,
            state,
            viewport,
            cursor_position,
            modifiers,
            clipboard,
            resized,
            debug,
            tex,
        } = self
        else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if *resized {
                    let size = window.inner_size();

                    *viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    *resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        Runner::render(
                            frame, device, state, viewport, scene, renderer, engine, queue, window,
                            debug, tex,
                        );
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
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
                renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                clipboard,
                debug,
            );

            // and request a redraw
            window.request_redraw();
        }
    }
}

impl Runner {
    fn render(
        frame: wgpu::SurfaceTexture,
        device: &wgpu::Device,
        state: &program::State<Controls>,
        viewport: &Viewport,
        scene: &Scene,
        renderer: &mut Renderer,
        engine: &mut Engine,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
        debug: &Debug,
        tex: &mut Arc<Mutex<Option<iced_wgpu::wgpu::Texture>>>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let texture_extent = wgpu::Extent3d {
            width: viewport.physical_width(),
            height: viewport.physical_height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            view_formats: &[frame.texture.format()],
            format: frame.texture.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            // We clear the texture
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw the scene to the texture
            scene.draw(&mut render_pass);
            let mut tex_guard = tex.lock().unwrap();
            *tex_guard = Some(texture);
        }

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            // We clear the frame
            // let mut render_pass = Scene::clear(&view, &mut encoder, Color::BLACK);

            // Draw the scene
            // scene.draw(&mut render_pass);
        }

        // And then iced on top
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
}
