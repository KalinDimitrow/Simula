pub use crate::rendering::liquid_crystal_latice::*;
pub use crate::rendering::vertex::*;
pub use iced::mouse;
pub use iced::widget::shader::{self, Viewport};
pub use iced::{Rectangle, Size};
pub use iced_wgpu::wgpu::util::DeviceExt;
pub use iced_wgpu::wgpu::IndexFormat;
pub use iced_wgpu::wgpu::TextureDescriptor;
pub use iced_wgpu::wgpu::*;

pub mod assets;
pub mod generic_pipeline;
pub mod liquid_crystal_latice;
pub mod renderers;
pub mod vertex;

use std::sync::{Arc, Mutex};

pub type TextureHandle = Arc<Mutex<Texture>>;
