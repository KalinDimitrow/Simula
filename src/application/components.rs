use crate::algorithm_processor::*;
use crate::application::{CustomEventProxy, SharedContext};
use crate::rendering::*;
use crate::rendering::renderers::BackgroundRenderer;
use crate::application::wininit_wrapper::WininitWrapper;
use crate::rendering::wgpu_wrapper::WGPUWrapper;
use crate::gui::controls::Controls;

pub struct Components {
    pub win: WininitWrapper,
    pub wgpu: WGPUWrapper,
    pub shared_context: SharedContext,
    pub algorithm_processor: AlgorithmProcessor,
    pub background_renderer: BackgroundRenderer,
    pub state: program::State<Controls>,
    pub event_proxy: CustomEventProxy,
    pub debug: Debug
}

impl Components {
    pub fn new(event_proxy: CustomEventProxy, event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let win = WininitWrapper::new(event_loop);
        let mut wgpu = WGPUWrapper::new(win.window.clone());
        let (data_handle, algorithm_processor) = AlgorithmProcessor::new(event_proxy.clone());
        let shared_context = SharedContext::new((300, 300));
        let background_renderer = BackgroundRenderer::new(&wgpu, &win.viewport, data_handle, shared_context.clone());
        let mut debug = Debug::new();
        let state = program::State::new(
            Controls::new(background_renderer.get_texture_handle(), event_proxy.clone()),
            win.viewport.logical_size(),
            &mut wgpu.renderer,
            &mut debug,
        );

        Self {
            win,
            wgpu,
            shared_context,
            algorithm_processor,
            background_renderer,
            state,
            event_proxy,
            debug
        }
    }
}
