use iced_wgpu::wgpu;

use iced::mouse;
use iced::time::Duration;
use iced::widget::shader::{self, Viewport};
use iced::{Color, Rectangle, Size};
use iced_wgpu::wgpu::IndexFormat;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

#[derive(Clone)]
pub struct Texture {
    tex: Arc<Mutex<Option<wgpu::Texture>>>,
}

impl Texture {
    pub fn new(tex: Arc<Mutex<Option<wgpu::Texture>>>) -> Self {
        Self { tex }
    }

    pub fn update(&mut self, time: Duration) {}
}

impl<Message> shader::Program<Message> for Texture {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(self.tex.clone()) // self.show_depth_buffer, self.light_color)
    }
}

/// A collection of `Cube`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
    tex: Arc<Mutex<Option<wgpu::Texture>>>,
    // uniforms: pipeline::Uniforms,
    // show_depth_buffer: bool,
}

impl Primitive {
    // pub fn new(bounds: Rectangle, show_depth_buffer: bool, light_color: Color) -> Self {
    pub fn new(tex: Arc<Mutex<Option<wgpu::Texture>>>) -> Self {
        // let uniforms = pipeline::Uniforms::new(camera, bounds, light_color);

        Self {
            tex,
            // uniforms,
            // show_depth_buffer,
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
                self.tex.clone(),
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Upload data to GPU
        // pipeline.update(
        //     device,
        //     queue,
        //     viewport.physical_size(),
        //     &self.uniforms,
        //     self.cubes.len(),
        //     &self.cubes,
        // );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Render primitive
        pipeline.render(target, encoder, *clip_bounds);
    }
}

const SKY_TEXTURE_SIZE: u32 = 128;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // D
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indicies: wgpu::Buffer,
    // tex: Arc<Mutex<Option<wgpu::Texture>>>,
    // uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
        tex: Arc<Mutex<Option<wgpu::Texture>>>,
    ) -> Self {
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texture vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indicies = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texture vertex buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // //uniforms for all cubes
        // let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("cubes uniform buffer"),
        //     size: std::mem::size_of::<Uniforms>() as u64,
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        let mut tex_guard = tex.lock().unwrap();
        let texture = tex_guard.as_ref().unwrap();

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cubes pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("texture shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shader/texture_shader.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            pipeline,
            // uniforms,
            bind_group,
            vertices,
            indicies,
        }
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
    ) {
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cubes.pipeline.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_scissor_rect(viewport.x, viewport.y, viewport.width, viewport.height);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_index_buffer(self.indicies.slice(..), IndexFormat::Uint16);
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
    }
}

struct DepthPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    depth_view: wgpu::TextureView,
}

impl DepthPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_texture: wgpu::TextureView,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cubes.depth_pipeline.sampler"),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cubes.depth_pipeline.bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cubes.depth_pipeline.bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_texture),
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cubes.depth_pipeline.layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cubes.depth_pipeline.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shader/texture_shader.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cubes.depth_pipeline.pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            sampler,
            depth_view: depth_texture,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, depth_texture: &wgpu::Texture) {
        self.depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cubes.depth_pipeline.bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.depth_view),
                },
            ],
        });
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        viewport: Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cubes.pipeline.depth_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: None,
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(viewport.x, viewport.y, viewport.width, viewport.height);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

pub struct Buffer {
    pub raw: wgpu::Buffer,
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
}

impl Buffer {
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        size: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        Self {
            raw: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            }),
            label,
            size,
            usage,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_size: u64) {
        if new_size > self.size {
            self.raw = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: new_size,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }
    }
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        //position
        0 => Float32x3,
        //normal
        1 => Float32x3,
        //tangent
        2 => Float32x3,
        //uv
        3 => Float32x2,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
