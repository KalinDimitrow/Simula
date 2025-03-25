use crate::algorithm_processor::*;
use crate::application::SharedContext;
use crate::rendering::wgpu_wrapper::WGPUWrapper;
use crate::rendering::*;

pub struct BackgroundRenderer {
    pub texture: TextureHandle,
    pub texture_view: TextureView,
    scene: Scene,
    data_handle: ProcessedDataHandle,
}

impl BackgroundRenderer {
    pub fn new(
        wgpu: &WGPUWrapper,
        viewport: &Viewport,
        data_handle: ProcessedDataHandle,
        shared_context: SharedContext,
    ) -> Self {
        let lattice_dimensions = { shared_context.lock().lattice_dimension };
        let scene = Scene::new(wgpu, lattice_dimensions);
        let texture_extent = Extent3d {
            width: viewport.physical_width(),
            height: viewport.physical_height(),
            depth_or_array_layers: 1,
        };

        let texture = wgpu.device.create_texture(&TextureDescriptor {
            label: Some("Render Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            view_formats: &[TextureFormat::Bgra8UnormSrgb],
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let texture = Arc::new(Mutex::new(texture));

        Self {
            texture,
            texture_view,
            scene,
            data_handle,
        }
    }

    pub fn render(&self, wgpu: &mut WGPUWrapper) {
        let mut encoder = wgpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut job_done = false;
        for datum in self.data_handle.try_iter() {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.scene.update(&wgpu.queue, datum);
            self.scene.draw(&mut render_pass);
            job_done = true;
        }

        if job_done {
            wgpu.engine.submit(&wgpu.queue, encoder);
        };
    }

    pub fn get_texture_handle(&self) -> TextureHandle {
        self.texture.clone()
    }
    pub fn resize_latice(&mut self, wgpu: &WGPUWrapper, lattice_dimensions: (usize, usize)) {
        self.scene = Scene::new(wgpu, lattice_dimensions);
    }
}
