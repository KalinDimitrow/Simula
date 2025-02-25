use crate::rendering::*;

const LATICE_SIZE: usize = 20;

fn generate_rectangle_vertices(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    center_x: f32,
    center_y: f32,
) -> Vec<OrbitingVertex> {
    vec![
        // First triangle of the rectangle
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            center: [center_x, center_y],
        }, // Bottom-left
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            center: [center_x, center_y],
        }, // Bottom-right
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            center: [center_x, center_y],
        }, // Top-left
        // Second triangle of the rectangle
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            center: [center_x, center_y],
        }, // Top-left
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            center: [center_x, center_y],
        }, // Bottom-right
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y1, 0.0],
                tex_coords: [1.0, 1.0],
            },
            center: [center_x, center_y],
        }, // Top-right
    ]
}

fn generate_vertex_buffer(rows: usize, cols: usize) -> Vec<OrbitingVertex> {
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

            vertices.extend(generate_rectangle_vertices(
                x0, y0, x1, y1, center_x, center_y,
            ));
        }
    }

    vertices
}

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
                generate_vertex_buffer(LATICE_SIZE, LATICE_SIZE).as_slice(),
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

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        use std::ops::Range;
        let range = Range::<u32> {
            start: 0,
            end: 6 * (LATICE_SIZE * LATICE_SIZE) as u32,
        };
        render_pass.draw(range, 0..1);
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(iced_wgpu::wgpu::include_wgsl!(
        "shader/2d_liquid_crystal_latice.wgsl"
    ));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render scene"),
        push_constant_ranges: &[],
        bind_group_layouts: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[OrbitingVertex::desc()],
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
