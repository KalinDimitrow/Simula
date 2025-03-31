use crate::rendering::*;
use rav1e::config::SpeedSettings;
use rav1e::prelude::*;
use std::fs::File;
use std::io::Write;

pub struct ImageWriter {
    ctx: Context<u8>,
    frames_count: usize,
}

impl ImageWriter {
    fn write_ivf_header<W: Write>(writer: &mut W, enc: &EncoderConfig, frame_count: u64) {
        let mut header = Vec::new();

        header.extend_from_slice(b"DKIF"); // signature
        header.extend_from_slice(&[0, 0]); // version
        header.extend_from_slice(&[32, 0]); // header size
        header.extend_from_slice(b"AV01"); // codec FourCC
        header.extend_from_slice(&(enc.width as u16).to_le_bytes()); // width
        header.extend_from_slice(&(enc.height as u16).to_le_bytes()); // height
        header.extend_from_slice(&30u32.to_le_bytes()); // timebase numerator (e.g., 1)
        header.extend_from_slice(&1u32.to_le_bytes()); // timebase denominator (e.g., 30 FPS)
        header.extend_from_slice(&(frame_count as u32).to_le_bytes()); // number of frames
        header.extend_from_slice(&[0, 0, 0, 0]); // unused

        writer.write_all(&header).unwrap();
    }

    fn write_ivf_packet<W: Write>(writer: &mut W, pkt: &Packet<u8>) {
        // packet size 12
        writer
            .write_all(&(pkt.data.len() as u32).to_le_bytes())
            .unwrap();
        writer
            .write_all(&(pkt.input_frameno as u64).to_le_bytes())
            .unwrap();
        writer.write_all(&pkt.data).unwrap();
    }

    fn rgb_to_yuv420(width: usize, height: usize, rgb: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut y_plane = vec![0u8; width * height];
        let mut u_plane = vec![0u8; (width * height) / 4];
        let mut v_plane = vec![0u8; (width * height) / 4];

        for j in 0..height {
            for i in 0..width {
                let r = rgb[3 * (j * width + i)];
                let g = rgb[3 * (j * width + i) + 1];
                let b = rgb[3 * (j * width + i) + 2];

                let y = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
                let u = ((-0.169 * r as f32 - 0.331 * g as f32 + 0.5 * b as f32) + 128.0) as u8;
                let v = ((0.5 * r as f32 - 0.419 * g as f32 - 0.081 * b as f32) + 128.0) as u8;

                y_plane[j * width + i] = y;

                if j % 2 == 0 && i % 2 == 0 {
                    u_plane[(j / 2) * (width / 2) + (i / 2)] = u;
                    v_plane[(j / 2) * (width / 2) + (i / 2)] = v;
                }
            }
        }

        (y_plane, u_plane, v_plane)
    }

    pub fn new(wgpu: &WGPUWrapper, frames_count: usize) -> Self {
        let enc = EncoderConfig {
            width: 640,
            height: 480,
            speed_settings: SpeedSettings::from_preset(3),
            time_base: Rational { num: 1, den: 30 },
            chroma_sampling: ChromaSampling::Cs420,
            ..Default::default()
        };

        let cfg = Config::new().with_encoder_config(enc.clone());

        let mut ctx: Context<u8> = cfg.new_context().unwrap();

        // let limit = 30;
        //
        // for i in 0..limit {
        //     println!("Sending frame {}", i);
        //     let mut f = ctx.new_frame();
        //
        //     // Create a colored gradient image for testing
        //     let mut rgb_pixels: Vec<u8> = Vec::with_capacity((enc.width * enc.height * 3) as usize);
        //     for y in 0..enc.height {
        //         for x in 0..enc.width {
        //             rgb_pixels.push(((x + i) % 256) as u8); // R
        //             rgb_pixels.push(((y + i) % 256) as u8); // G
        //             rgb_pixels.push(((x + y) % 256) as u8); // B
        //         }
        //     }
        //
        //     let (y_plane, u_plane, v_plane) =
        //         ImageWriter::rgb_to_yuv420(enc.width as usize, enc.height as usize, &rgb_pixels);
        //
        //     f.planes[0].copy_from_raw_u8(&y_plane, enc.width, 1);
        //     f.planes[1].copy_from_raw_u8(&u_plane, enc.width / 2, 1);
        //     f.planes[2].copy_from_raw_u8(&v_plane, enc.width / 2, 1);
        //     match ctx.send_frame(f.clone()) {
        //         Ok(_) => {}
        //         Err(e) => match e {
        //             EncoderStatus::EnoughData => {
        //                 println!("Unable to append frame {} to the internal queue", i);
        //             }
        //             _ => {
        //                 panic!("Unable to send frame {}", i);
        //             }
        //         },
        //     }
        // }
        //
        // ctx.flush();
        // let mut file = File::create("output.ivf").unwrap();
        // ImageWriter::write_ivf_header(&mut file, &enc, limit as u64);
        //
        // // Test that we cleanly exit once we hit the limit
        // let mut i = 0;
        // while i < limit + 5 {
        //     match ctx.receive_packet() {
        //         Ok(pkt) => {
        //             ImageWriter::write_ivf_packet(&mut file, &pkt);
        //             println!("Packet {}", pkt.input_frameno);
        //             i += 1;
        //         }
        //         Err(e) => match e {
        //             EncoderStatus::LimitReached => {
        //                 println!("Limit reached");
        //                 break;
        //             }
        //             EncoderStatus::Encoded => println!("  Encoded"),
        //             EncoderStatus::NeedMoreData => println!("  Need more data"),
        //             _ => {
        //                 panic!("Unable to receive packet {}", i);
        //             }
        //         },
        //     }
        // }
        // file.flush().unwrap();
        Self { ctx, frames_count }
    }

    pub fn write_image(
        &self,
        wgpu: &WGPUWrapper,
        texture: &wgpu::Texture,
        dimensions: (u32, u32),
    ) -> Vec<u8> {
        let device = &wgpu.device;
        let queue = &wgpu.queue;
        let buffer_size = (dimensions.0 * dimensions.1 * 4) as wgpu::BufferAddress;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Copy Encoder"),
        });

        // Copy the texture to the output buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(dimensions.0 * 4),
                    rows_per_image: Some(dimensions.1),
                },
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        // Map the buffer and read data
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        // Copy the data out of the buffer
        let data = buffer_slice.get_mapped_range().to_vec();
        output_buffer.unmap();

        data
    }
}
