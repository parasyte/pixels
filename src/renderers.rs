use ultraviolet::Mat4;
use wgpu::*;

use crate::include_spv;

/// Renderer implements [`RenderPass`].
#[derive(Debug)]
pub(crate) struct ScalingRenderer {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    width: f32,
    height: f32,
}

impl ScalingRenderer {
    /// Factory function for generating `RenderPass` trait objects.
    pub(crate) fn new(
        device: &mut Device,
        texture_view: &TextureView,
        texture_size: &Extent3d,
    ) -> Self {
        let vs_module = device.create_shader_module(include_spv!("../shaders/vert.spv"));
        let fs_module = device.create_shader_module(include_spv!("../shaders/frag.spv"));

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: CompareFunction::Always,
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
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        );

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        component_type: TextureComponentType::Uint,
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..64,
                    },
                },
            ],
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
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

    pub fn render(&self, encoder: &mut CommandEncoder, render_target: &TextureView) {
        // Draw the updated texture to the render target
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_color: Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }

    pub fn resize(
        &mut self,
        device: &mut Device,
        encoder: &mut CommandEncoder,
        width: u32,
        height: u32,
    ) {
        let matrix = ScalingMatrix::new((self.width, self.height), (width as f32, height as f32));
        let transform_bytes = matrix.as_bytes();

        let temp_buf = device.create_buffer_with_data(&transform_bytes, BufferUsage::COPY_SRC);
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
