use crate::algorithm_processor::*;
use crate::application::SharedContext;
use crate::rendering::*;
use crate::rendering::webgpu_wrapper::WebGPUWrapper;

pub struct BackgroundRenderer {
    pub texture: TextureHandle,
    pub texture_view: TextureView,
    scene: Scene,
    data_handle: ProcessedDataHandle,
}

impl BackgroundRenderer {
    pub fn new(
        wgpu_wrapper: &WebGPUWrapper,
        // device: &Device,
        // queue: &Queue,
        viewport: &Viewport,
        data_handle: ProcessedDataHandle,
        shared_context: SharedContext,
    ) -> Self {
        let latice_dimentions = { shared_context.lock().latice_dimentions };
        let scene = Scene::new(wgpu_wrapper, latice_dimentions);
        let texture_extent = Extent3d {
            width: viewport.physical_width(),
            height: viewport.physical_height(),
            depth_or_array_layers: 1,
        };

        let texture = wgpu_wrapper.device.create_texture(&TextureDescriptor {
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

    pub fn render(&self, web_gpuwrapper: &mut WebGPUWrapper) {
        let mut encoder = web_gpuwrapper.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
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

            self.scene.update(&web_gpuwrapper.queue, datum);
            self.scene.draw(&mut render_pass);
            job_done = true;
        }

        if job_done {
            web_gpuwrapper.engine.submit(&web_gpuwrapper.queue, encoder);
        };
    }

    pub fn get_texture_handle(&self) -> TextureHandle {
        self.texture.clone()
    }
}
