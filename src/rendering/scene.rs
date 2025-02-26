use crate::rendering::assets::*;
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
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Scene {
    pub fn new(device: &Device, queue: &Queue, texture_format: TextureFormat) -> Scene {
        let (pipeline, bind_group) = build_pipeline(device, queue, texture_format);
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(
                generate_vertex_buffer(LATICE_SIZE, LATICE_SIZE).as_slice(),
            ),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        Scene {
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
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
    device: &Device,
    queue: &Queue,
    texture_format: TextureFormat,
) -> (RenderPipeline, BindGroup) {
    let shader = device.create_shader_module(iced_wgpu::wgpu::include_wgsl!(
        "shader/2d_liquid_crystal_latice.wgsl"
    ));

    let texture = create_texture_from_image(device, queue, IMAGE_DATA);
    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let sampler = device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("texture bind group layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("texture_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Render scene"),
        push_constant_ranges: &[],
        bind_group_layouts: &[&texture_bind_group_layout],
    });

    (
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[OrbitingVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        }),
        bind_group,
    )
}
