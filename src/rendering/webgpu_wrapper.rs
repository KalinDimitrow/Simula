use std::sync::Arc;
pub use iced_wgpu::*;
use crate::rendering::*;
use crate::rendering::webgpu_wrapper::graphics::Antialiasing;

pub struct WebGPUWrapper {
    pub backends: Backends,
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub format: TextureFormat,
    pub engine: Engine,
    pub renderer: Renderer,
}

impl WebGPUWrapper {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let backends = util::backend_bits_from_env().unwrap_or_default();
        let instance = Self::create_instance(backends);
        let mut surface = Self::create_surface(&instance, window.clone());
        let (format, adapter, device, queue) = Self::get_gpu_bits(&instance, &surface);
        let engine = Engine::new(
            &adapter,
            &device,
            &queue,
            format,
            Some(Antialiasing::MSAAx4),
        );
        let renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

        Self::configure_surface(&mut surface, &device, &*window, format);
        Self {
            backends,
            instance,
            device,
            queue,
            surface,
            engine,
            renderer,
            format,
        }
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
}