use crate::rendering::*;

pub struct BackgroundRenderer {
    pub texture: TextureHandle,
    pub texture_view: TextureView,
    scene: Scene,
}

impl BackgroundRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        viewport: &Viewport,
        texture_format: TextureFormat,
    ) -> Self {
        let scene = Scene::new(device, queue, texture_format);
        let texture_extent = Extent3d {
            width: viewport.physical_width(),
            height: viewport.physical_height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
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
        }
    }

    pub fn render<'a>(&'a self, encoder: &mut CommandEncoder) {
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

        self.scene.draw(&mut render_pass);
    }

    pub fn get_texture_handle(&self) -> TextureHandle {
        self.texture.clone()
    }
}
