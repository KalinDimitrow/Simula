use crate::algorithm_processor::*;
use crate::application::wininit_wrapper::WininitWrapper;
use crate::application::{CustomEventProxy, SharedContext, AlgorithmCatalog};
use crate::gui::controls::Controls;
use crate::rendering::ImageWriter;
use crate::rendering::renderers::BackgroundRenderer;
use crate::rendering::wgpu_wrapper::WGPUWrapper;
use crate::rendering::*;

pub struct Components {
    pub win: WininitWrapper,
    pub wgpu: WGPUWrapper,
    pub shared_context: SharedContext,
    pub algorithm_processor: AlgorithmProcessor,
    pub background_renderer: BackgroundRenderer,
    pub state: program::State<Controls>,
    pub image_writer: ImageWriter,
    pub event_proxy: CustomEventProxy,
    pub algorithm_catalog: AlgorithmCatalog,
    pub debug: Debug,
}

impl Components {
    pub fn new(
        event_proxy: CustomEventProxy,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Self {
        let win = WininitWrapper::new(event_loop);
        let mut wgpu = WGPUWrapper::new(win.window.clone());
        let shared_context = SharedContext::new(event_proxy.clone(), (300, 300));
        let (data_handle, algorithm_processor) = AlgorithmProcessor::new(shared_context.clone());
        let background_renderer =
            BackgroundRenderer::new(&wgpu, &win.viewport, data_handle, shared_context.clone());
        let mut debug = Debug::new();
        let state = program::State::new(
            Controls::new(
                background_renderer.get_texture_handle(),
                event_proxy.clone(),
            ),
            win.viewport.logical_size(),
            &mut wgpu.renderer,
            &mut debug,
        );

        let image_writer = ImageWriter::new(&wgpu, 5);

        let algorithm_catalog = AlgorithmCatalog::new();

        Self {
            win,
            wgpu,
            shared_context,
            algorithm_processor,
            background_renderer,
            state,
            image_writer,
            event_proxy,
            algorithm_catalog,
            debug,
        }
    }
}
