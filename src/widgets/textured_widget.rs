use crate::rendering::generic_pipeline::Pipeline;
use crate::rendering::*;

#[derive(Clone)]
pub struct TexturedWidget {
    texture: TextureHandle,
}

impl TexturedWidget {
    pub fn new(texture: TextureHandle) -> Self {
        Self { texture }
    }
}

impl<Message> shader::Program<Message> for TexturedWidget {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(self.texture.clone())
    }
}

#[derive(Debug)]
pub struct Primitive {
    texture: TextureHandle,
}

impl Primitive {
    pub fn new(texture: TextureHandle) -> Self {
        Self { texture }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &iced_wgpu::wgpu::Device,
        queue: &iced_wgpu::wgpu::Queue,
        format: iced_wgpu::wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
                self.texture.clone(),
            ));
        }
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        storage: &shader::Storage,
        target: &TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Render primitive
        pipeline.render(target, encoder, *clip_bounds);
    }
}
