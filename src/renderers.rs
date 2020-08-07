use crate::include_spv;
use ultraviolet::Mat4;

/// The default renderer that scales your frame to the screen size.
#[derive(Debug)]
pub struct ScalingRenderer {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    width: f32,
    height: f32,
}

impl ScalingRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        texture_size: &wgpu::Extent3d,
    ) -> Self {
        let vs_module = device.create_shader_module(include_spv!("../shaders/vert.spv"));
        let fs_module = device.create_shader_module(include_spv!("../shaders/frag.spv"));

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: wgpu::CompareFunction::Always,
        });

        // Create uniform buffer
        // TODO: This should also have the width / height of the of the window surface,
        // so that it won't break when the window is created with a different size.
        let matrix = ScalingMatrix::new(
            (texture_size.width as f32, texture_size.height as f32),
            (texture_size.width as f32, texture_size.height as f32),
        );
        let transform_bytes = matrix.as_bytes();
        let uniform_buffer = device.create_buffer_with_data(
            &transform_bytes,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Uint,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..64,
                    },
                },
            ],
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            uniform_buffer,
            bind_group,
            render_pipeline,
            width: texture_size.width as f32,
            height: texture_size.height as f32,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, render_target: &wgpu::TextureView) {
        // Draw the updated texture to the render target
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }

    pub(crate) fn resize(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
    ) {
        let matrix = ScalingMatrix::new((self.width, self.height), (width as f32, height as f32));
        let transform_bytes = matrix.as_bytes();

        let temp_buf =
            device.create_buffer_with_data(&transform_bytes, wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buffer, 0, 64);
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
