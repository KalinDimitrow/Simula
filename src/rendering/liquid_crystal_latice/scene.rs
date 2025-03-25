use super::scene_generation::generate_vertex_buffer;
use crate::algorithm_processor::*;
use crate::rendering::assets::*;
use crate::rendering::wgpu_wrapper::WGPUWrapper;
use crate::rendering::*;

type Dimentions = (usize, usize);

pub struct Scene {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    _uniform_buffer: Buffer,
    storage_buffer: Buffer,
    _angle_data: Data,
    dimentions: (usize, usize),
}

impl Scene {
    pub fn new(webgpu_wrapper: &WGPUWrapper, dimentions: Dimentions) -> Scene {
        let _angle_data = vec![0.0; dimentions.0 * dimentions.1];
        let (pipeline, bind_group, _uniform_buffer, storage_buffer) = build_pipeline(
            &webgpu_wrapper.device,
            &webgpu_wrapper.queue,
            TextureFormat::Bgra8UnormSrgb,
            &dimentions,
            &_angle_data,
        );
        let vertex_buffer = webgpu_wrapper
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(
                    generate_vertex_buffer(dimentions.0, dimentions.1).as_slice(),
                ),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = webgpu_wrapper
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX,
            });

        Scene {
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            _uniform_buffer,
            storage_buffer,
            _angle_data,
            dimentions,
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
            end: 6 * (self.dimentions.0 * self.dimentions.1) as u32,
        };
        render_pass.draw(range, 0..1);
    }

    pub fn update(&self, queue: &Queue, angle: Data) {
        queue.write_buffer(&self.storage_buffer, 0, bytemuck::cast_slice(&angle));
    }
}

fn build_pipeline(
    device: &Device,
    queue: &Queue,
    texture_format: TextureFormat,
    dimentions: &Dimentions,
    angle_data: &Vec<f32>,
) -> (RenderPipeline, BindGroup, Buffer, Buffer) {
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
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let storage_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(&angle_data.as_slice()),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    let uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some("Single Value Buffer"),
        contents: bytemuck::cast_slice(&[dimentions.0 as f32, dimentions.1 as f32]),
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
                resource: uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: storage_buffer.as_entire_binding(),
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
        uniform_buffer,
        storage_buffer,
    )
}
