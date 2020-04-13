use std::fmt;
use std::rc::Rc;
use wgpu::{self, Extent3d, TextureView};

use crate::include_spv;
use crate::render_pass::{BoxedRenderPass, Device, Queue, RenderPass};

/// Renderer implements [`RenderPass`].
#[derive(Debug)]
pub(crate) struct Renderer {
    device: Rc<wgpu::Device>,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    width: f32,
    height: f32,
}

impl Renderer {
    /// Factory function for generating `RenderPass` trait objects.
    pub(crate) fn factory(
        device: Device,
        _queue: Queue,
        texture_view: &TextureView,
        texture_size: &Extent3d,
    ) -> BoxedRenderPass {
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
        // TODO: Figure out how to store floats for this transform if necessary
        #[rustfmt::skip]
        let mut transform: [u8; 16] = [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let mut uniform_buffer = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        uniform_buffer.data = &mut transform;
        let uniform_buffer = uniform_buffer.finish();

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        // TODO: Make sure this is right component type
                        component_type: wgpu::TextureComponentType::Uint,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        // TODO: Make sure this is the right value
                        comparison: false,
                    },
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

        Box::new(Renderer {
            device,
            uniform_buffer,
            bind_group,
            render_pipeline,
            width: texture_size.width as f32,
            height: texture_size.height as f32,
        })
    }
}

impl RenderPass for Renderer {
    fn render(&self, encoder: &mut wgpu::CommandEncoder, render_target: &TextureView) {
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

    fn resize(&mut self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        let width = width as f32;
        let height = height as f32;

        // Get smallest scale size
        let scale = (width / self.width)
            .min(height / self.height)
            .max(1.0)
            .floor();

        // Update transformation matrix
        let sw = self.width * scale / width;
        let sh = self.height * scale / height;
        // TODO: Figure out how to encode floats in the transform buffer
        #[rustfmt::skip]
        let mut transform: [u8; 16] = [
            sw as u8, 0, 0, 0,
            0, sh as u8, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];

        let mut temp_buf = self.device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        temp_buf.data = &mut transform;
        let temp_buf = temp_buf.finish();
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buffer, 0, 64);
    }

    // We don't actually have to rebind the TextureView here.
    // It's guaranteed that the initial texture never changes.
    fn update_bindings(&mut self, _input_texture: &TextureView, _input_texture_size: &Extent3d) {}

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
