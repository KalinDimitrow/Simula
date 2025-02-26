use crate::rendering::generic_pipeline::Pipeline;
use crate::rendering::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TexturedWidget {
    tex: Arc<Mutex<Option<Texture>>>,
}

impl TexturedWidget {
    pub fn new(tex: Arc<Mutex<Option<Texture>>>) -> Self {
        Self { tex }
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
        Primitive::new(self.tex.clone())
    }
}

#[derive(Debug)]
pub struct Primitive {
    tex: Arc<Mutex<Option<Texture>>>,
}

impl Primitive {
    pub fn new(tex: Arc<Mutex<Option<Texture>>>) -> Self {
        Self { tex }
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
                self.tex.clone(),
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
