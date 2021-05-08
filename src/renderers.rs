use crate::SurfaceSize;
use ultraviolet::Mat4;
use wgpu::util::DeviceExt;

/// The default renderer that scales your frame to the screen size.
#[derive(Debug)]
pub struct ScalingRenderer {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    width: f32,
    height: f32,
    render_texture_format: wgpu::TextureFormat,
}

impl ScalingRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        texture_size: &wgpu::Extent3d,
        surface_size: &SurfaceSize,
        render_texture_format: wgpu::TextureFormat,
    ) -> Self {
        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../shaders/vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../shaders/frag.spv"));

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pixels_scaling_renderer_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        // Create uniform buffer
        let matrix = ScalingMatrix::new(
            (texture_size.width as f32, texture_size.height as f32),
            (surface_size.width as f32, surface_size.height as f32),
        );
        let transform_bytes = matrix.as_bytes();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("pixels_scaling_renderer_matrix_uniform_buffer"),
            contents: &transform_bytes,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pixels_scaling_renderer_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pixels_scaling_renderer_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pixels_scaling_renderer_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pixels_scaling_renderer_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: render_texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        Self {
            uniform_buffer,
            bind_group,
            render_pipeline,
            width: texture_size.width as f32,
            height: texture_size.height as f32,
            render_texture_format,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, render_target: &wgpu::TextureView) {
        // Draw the updated texture to the render target
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pixels_scaling_renderer_render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }

    pub(crate) fn resize(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        let matrix = ScalingMatrix::new((self.width, self.height), (width as f32, height as f32));
        let transform_bytes = matrix.as_bytes();
        queue.write_buffer(&self.uniform_buffer, 0, &transform_bytes);
    }
}

#[derive(Debug)]
pub(crate) struct ScalingMatrix {
    pub(crate) transform: Mat4,
}

impl ScalingMatrix {
    // texture_size is the dimensions of the drawing texture
    // screen_size is the dimensions of the surface being drawn to
    pub(crate) fn new(texture_size: (f32, f32), screen_size: (f32, f32)) -> ScalingMatrix {
        let (screen_width, screen_height) = screen_size;
        let (texture_width, texture_height) = texture_size;

        // Get smallest scale size
        let scale = (screen_width / texture_width)
            .min(screen_height / texture_height)
            .max(1.0)
            .floor();

        // Update transformation matrix
        let sw = texture_width * scale / screen_width;
        let sh = texture_height * scale / screen_height;
        #[rustfmt::skip]
        let transform: [f32; 16] = [
            sw,  0.0, 0.0, 0.0,
            0.0, -sh, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        ScalingMatrix {
            transform: Mat4::from(transform),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        self.transform.as_byte_slice()
    }
}
