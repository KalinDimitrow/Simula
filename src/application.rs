use crate::algorithm_processor::{self, *};
use crate::gui::controls::Controls;
use crate::rendering::renderers::*;
use crate::rendering::*;
use winit::event_loop::EventLoopProxy;

use std::sync::Arc;

use winit::{event::WindowEvent, event_loop::ControlFlow, keyboard::ModifiersState};

#[allow(clippy::large_enum_variant)]
enum WindowContext {
    Loading,
    Ready {
        window: Arc<winit::window::Window>,
        device: Device,
        queue: Queue,
        surface: Surface<'static>,
        format: TextureFormat,
        engine: Engine,
        renderer: Renderer,
        cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
        clipboard: Clipboard,
        viewport: Viewport,
        modifiers: ModifiersState,
        resized: bool,
        state: program::State<Controls>,
    },
}
pub struct Simula {
    ctx: WindowContext,
    event_proxy: EventLoopProxy<()>,
    algorithm_processor: Option<AlgorithmProcessor>,
    background_renderer: Option<BackgroundRenderer>,
    debug: Debug,
}

impl Simula {
    pub fn new(event_proxy: EventLoopProxy<()>) -> Self {
        Self {
            ctx: WindowContext::Loading,
            event_proxy,
            algorithm_processor: None,
            background_renderer: None,
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
        let window = Arc::new(
            event_loop
                .create_window(winit::window::WindowAttributes::default())
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

    fn create_instance(backend: Backends) -> Instance {
        Instance::new(InstanceDescriptor {
            backends: backend,
            ..Default::default()
        })
    }

    fn create_surface<'a>(instance: &Instance, window: Arc<winit::window::Window>) -> Surface<'a> {
        instance
            .create_surface(window.clone())
            .expect("Create window surface")
    }

    fn get_gpu_bits(
        instance: &Instance,
        surface: &Surface,
    ) -> (TextureFormat, Adapter, Device, Queue) {
        futures::futures::executor::block_on(async {
            let adapter = util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                .await
                .expect("Create adapter");

            let adapter_features = adapter.features();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        label: None,
                        required_features: adapter_features & Features::default(),
                        required_limits: Limits::default(),
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
                    .find(TextureFormat::is_srgb)
                    .or_else(|| capabilities.formats.first().copied())
                    .expect("Get preferred format"),
                adapter,
                device,
                queue,
            )
        })
    }

    fn configure_surface(
        surface: &mut Surface,
        device: &Device,
        window: &winit::window::Window,
        format: TextureFormat,
    ) {
        let physical_size = window.inner_size();
        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: PresentMode::AutoVsync,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );
    }

    fn handle_redraw_event(
        resized: &mut bool,
        window: &winit::window::Window,
        viewport: &mut Viewport,
        surface: &mut Surface,
        device: &Device,
        format: &TextureFormat,
        state: &program::State<Controls>,
        renderer: &mut Renderer,
        engine: &mut Engine,
        queue: &Queue,
        debug: &Debug,
    ) {
        if *resized {
            let size = window.inner_size();

            *viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                window.scale_factor(),
            );

            surface.configure(
                device,
                &SurfaceConfiguration {
                    format: *format,
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

        match surface.get_current_texture() {
            Ok(frame) => {
                Simula::render(
                    frame, device, state, viewport, renderer, engine, queue, window, &debug,
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

impl winit::application::ApplicationHandler for Simula {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let WindowContext::Loading = self.ctx {
            let window = Self::create_window(event_loop);
            let viewport = Self::get_viewport(&*window);
            let clipboard = Clipboard::connect(window.clone());
            let backend = util::backend_bits_from_env().unwrap_or_default();
            let instance = Self::create_instance(backend);
            let mut surface = Self::create_surface(&instance, window.clone());
            let (format, adapter, device, queue) = Self::get_gpu_bits(&instance, &surface);
            Self::configure_surface(&mut surface, &device, &*window, format);

            let (data_handle, _algorithm_processor) =
                algorithm_processor::AlgorithmProcessor::new();
            let background_renderer =
                BackgroundRenderer::new(&device, &queue, &viewport, data_handle);

            let engine = Engine::new(&adapter, &device, &queue, format, None);
            let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

            let state = program::State::new(
                Controls::new(background_renderer.get_texture_handle()),
                viewport.logical_size(),
                &mut renderer,
                &mut self.debug,
            );

            self.background_renderer = Some(background_renderer);
            self.algorithm_processor = Some(_algorithm_processor);

            event_loop.set_control_flow(ControlFlow::Wait);

            self.ctx = WindowContext::Ready {
                window,
                device,
                queue,
                surface,
                format,
                engine,
                renderer,
                cursor_position: None,
                modifiers: ModifiersState::default(),
                clipboard,
                viewport,
                resized: false,
                state,
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let WindowContext::Ready {
            window,
            device,
            queue,
            surface,
            format,
            engine,
            renderer,
            viewport,
            cursor_position,
            modifiers,
            clipboard,
            resized,
            state,
        } = &mut self.ctx
        else {
            return;
        };

        if let Some(background_renderer) = self.background_renderer.as_ref() {
            background_renderer.render(device, &queue, engine);
        }

        match event {
            WindowEvent::RedrawRequested => {
                Simula::handle_redraw_event(
                    resized,
                    window,
                    viewport,
                    surface,
                    device,
                    format,
                    state,
                    renderer,
                    engine,
                    queue,
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
                renderer,
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

        window.request_redraw();
    }
}
