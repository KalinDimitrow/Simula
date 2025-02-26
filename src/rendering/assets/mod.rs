use image::GenericImageView;
use std::io::Cursor;

use crate::rendering::*;
pub mod textures;
pub use crate::rendering::assets::textures::*;

pub fn create_texture_from_image(device: &Device, queue: &Queue, data: &[u8]) -> Texture {
    let (data, width, height) = load_image(data);

    let texture_size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("Render Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    });

    // Upload the pixel data to the texture
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    texture
}

fn load_image(data: &[u8]) -> (Vec<u8>, u32, u32) {
    let img =
        image::load(Cursor::new(data), image::ImageFormat::Png).expect("Failed to load image");
    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();
    (rgba.into_raw(), dimensions.0, dimensions.1)
}
