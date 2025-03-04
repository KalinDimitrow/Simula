pub use crate::rendering::liquid_crystal_latice::*;
pub use crate::rendering::vertex::*;
pub use iced::mouse;
pub use iced::widget::shader::{self, Viewport};
pub use iced::{Rectangle, Size};
pub use iced_wgpu::wgpu::util::DeviceExt;
pub use iced_wgpu::wgpu::IndexFormat;
pub use iced_wgpu::wgpu::TextureDescriptor;
pub use iced_wgpu::wgpu::*;
pub use iced_wgpu::{Engine, Renderer};
pub use iced_winit::conversion;
pub use iced_winit::core::renderer;
pub use iced_winit::core::{Color, Font, Pixels, Theme};
pub use iced_winit::futures;
pub use iced_winit::runtime::program;
pub use iced_winit::runtime::Debug;
pub use iced_winit::winit;
pub use iced_winit::Clipboard;
pub mod assets;
pub mod generic_pipeline;
pub mod liquid_crystal_latice;
pub mod renderers;
pub mod vertex;

use std::sync::{Arc, Mutex};

pub type TextureHandle = Arc<Mutex<Texture>>;
