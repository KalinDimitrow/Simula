use iced_wgpu::wgpu;
use iced_winit::core::Color;
use std::iter;
use wgpu::util::DeviceExt;
use wgpu_types::*;
const latice_size: usize = 20;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    center: [f32; 2],
    // tex_coords: [f32; 2],
}

pub fn generate_vertex_buffer(rows: usize, cols: usize) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(rows * cols * 6);

    let row_step = 2.0 / rows as f32;
    let col_step = 2.0 / cols as f32;
    let spacing_x = col_step / 3.0;
    let spacing_y = row_step / 3.0;

    for row in 0..rows {
        let y0 = -1.0 + row as f32 * row_step + spacing_y / 2.0;
        let y1 = y0 + row_step - spacing_y;
        let center_y = (y0 + y1) / 2.0;

        for col in 0..cols {
            let x0 = -1.0 + col as f32 * col_step + spacing_x / 2.0;
            let x1 = x0 + col_step - spacing_x;
            let center_x = (x0 + x1) / 2.0;

            // First triangle of the rectangle
            vertices.push(Vertex {
                position: [x0, y0, 0.0],
                center: [center_x, center_y],
            }); // Bottom-left
            vertices.push(Vertex {
                position: [x1, y0, 0.0],
                center: [center_x, center_y],
            }); // Bottom-right
            vertices.push(Vertex {
                position: [x0, y1, 0.0],
                center: [center_x, center_y],
            }); // Top-left

            // Second triangle of the rectangle
            vertices.push(Vertex {
                position: [x0, y1, 0.0],
                center: [center_x, center_y],
            }); // Top-left
            vertices.push(Vertex {
                position: [x1, y0, 0.0],
                center: [center_x, center_y],
            }); // Bottom-right
            vertices.push(Vertex {
                position: [x1, y1, 0.0],
                center: [center_x, center_y],
            }); // Top-right
        }
    }

    vertices
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct Scene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Scene {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Scene {
        let pipeline = build_pipeline(device, texture_format);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(
                generate_vertex_buffer(latice_size, latice_size).as_slice(),
            ),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Scene {
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn clear<'a>(
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
        background_color: Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let [r, g, b, a] = background_color.into_linear();

                        wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        use std::ops::Range;
        let range = Range::<u32> {
            start: 0,
            end: 6 * (latice_size * latice_size) as u32,
        };
        render_pass.draw(range, 0..1);
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        push_constant_ranges: &[],
        bind_group_layouts: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
