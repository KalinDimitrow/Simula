use super::scene_generation::generate_vertex_buffer;
use crate::rendering::assets::*;
use crate::rendering::*;

const LATICE_SIZE: usize = 20;

pub struct Scene {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    angle_buffer: Buffer,
}

impl Scene {
    pub fn new(device: &Device, queue: &Queue, texture_format: TextureFormat) -> Scene {
        let (pipeline, bind_group, angle_buffer) = build_pipeline(device, queue, texture_format);
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
            angle_buffer,
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

    pub fn update(&self, queue: &Queue, angle: f32) {
        queue.write_buffer(&self.angle_buffer, 0, bytemuck::cast_slice(&[angle]));
    }
}

fn build_pipeline(
    device: &Device,
    queue: &Queue,
    texture_format: TextureFormat,
) -> (RenderPipeline, BindGroup, Buffer) {
    let shader = device.create_shader_module(iced_wgpu::wgpu::include_wgsl!(
        "../shader/2d_liquid_crystal_latice.wgsl"
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
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let angle_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some("Single Value Buffer"),
        contents: bytemuck::cast_slice(&[0]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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
            BindGroupEntry {
                binding: 2,
                resource: angle_buffer.as_entire_binding(),
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
        angle_buffer,
    )
}
