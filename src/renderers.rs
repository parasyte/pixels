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
        let matrix = ScalingMatrix::new((self.width, self.height), (width as f32, height as f32));
        let transform_bytes = matrix.as_bytes();

        let temp_buf = self
            .device
            .create_buffer_with_data(&transform_bytes, wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buffer, 0, 64);
    }

    // We don't actually have to rebind the TextureView here.
    // It's guaranteed that the initial texture never changes.
    fn update_bindings(&mut self, _input_texture: &TextureView, _input_texture_size: &Extent3d) {}

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub(crate) struct ScalingMatrix {
    pub(crate) transform: [[f32; 4]; 4],
}

impl ScalingMatrix {
    // texture_size is the dimensions of the drawing texture
    // screen_size is the dimensions of the surface being drawn to
    pub(crate) fn new(texture_size: (f32, f32), screen_size: (f32, f32)) -> ScalingMatrix {
        let screen_width = screen_size.0;
        let screen_height = screen_size.1;
        let texture_width = texture_size.0;
        let texture_height = texture_size.1;

        // Get smallest scale size
        let scale = (screen_width / texture_width)
            .min(screen_height / texture_height)
            .max(1.0)
            .floor();

        // Update transformation matrix
        let sw = texture_width * scale / screen_width;
        let sh = texture_height * scale / screen_height;
        #[rustfmt::skip]
        let transform: [[f32; 4]; 4] = [
            [sw,  0.0, 0.0, 0.0],
            [0.0, -sh, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        ScalingMatrix { transform }
    }

    // Could be done with unsafe code or a library like bytemuck, but that
    // shouldn't be needed as this is rarely called (only on creation / resize).
    fn as_bytes(&self) -> [u8; 4 * 4 * 4] {
        let mut transform_bytes = [0; 4 * 4 * 4];
        let mut i = 0;
        for row in self.transform.iter() {
            for f in row.iter() {
                for b in f.to_bits().to_ne_bytes().iter() {
                    transform_bytes[i] = *b;
                    i += 1;
                }
            }
        }
        transform_bytes
    }

    // Calculate the inverse of the 4x4 matrix
    // This is a ported version of https://stackoverflow.com/questions/1148309/inverting-a-4x4-matrix#answer-44446912
    // This should probably use a library instead, but it isn't worth
    // adding a large dependency for one function
    #[rustfmt::skip]
    pub(crate) fn inverse(&self) -> [[f32; 4]; 4] {
        let m = &self.transform;
        let a2323 = m[2][2] * m[3][3] - m[2][3] * m[3][2];
        let a1323 = m[2][1] * m[3][3] - m[2][3] * m[3][1];
        let a1223 = m[2][1] * m[3][2] - m[2][2] * m[3][1];
        let a0323 = m[2][0] * m[3][3] - m[2][3] * m[3][0];
        let a0223 = m[2][0] * m[3][2] - m[2][2] * m[3][0];
        let a0123 = m[2][0] * m[3][1] - m[2][1] * m[3][0];
        let a2313 = m[1][2] * m[3][3] - m[1][3] * m[3][2];
        let a1313 = m[1][1] * m[3][3] - m[1][3] * m[3][1];
        let a1213 = m[1][1] * m[3][2] - m[1][2] * m[3][1];
        let a2312 = m[1][2] * m[2][3] - m[1][3] * m[2][2];
        let a1312 = m[1][1] * m[2][3] - m[1][3] * m[2][1];
        let a1212 = m[1][1] * m[2][2] - m[1][2] * m[2][1];
        let a0313 = m[1][0] * m[3][3] - m[1][3] * m[3][0];
        let a0213 = m[1][0] * m[3][2] - m[1][2] * m[3][0];
        let a0312 = m[1][0] * m[2][3] - m[1][3] * m[2][0];
        let a0212 = m[1][0] * m[2][2] - m[1][2] * m[2][0];
        let a0113 = m[1][0] * m[3][1] - m[1][1] * m[3][0];
        let a0112 = m[1][0] * m[2][1] - m[1][1] * m[2][0];

        let mut det =
              m[0][0] * (m[1][1] * a2323 - m[1][2] * a1323 + m[1][3] * a1223)
            - m[0][1] * (m[1][0] * a2323 - m[1][2] * a0323 + m[1][3] * a0223)
            + m[0][2] * (m[1][0] * a1323 - m[1][1] * a0323 + m[1][3] * a0123)
            - m[0][3] * (m[1][0] * a1223 - m[1][1] * a0223 + m[1][2] * a0123);

        det = 1.0 / det;

        [
            [
                det *  (m[1][1] * a2323 - m[1][2] * a1323 + m[1][3] * a1223),
                det * -(m[0][1] * a2323 - m[0][2] * a1323 + m[0][3] * a1223),
                det *  (m[0][1] * a2313 - m[0][2] * a1313 + m[0][3] * a1213),
                det * -(m[0][1] * a2312 - m[0][2] * a1312 + m[0][3] * a1212),
            ],
            [
                det * -(m[1][0] * a2323 - m[1][2] * a0323 + m[1][3] * a0223),
                det *  (m[0][0] * a2323 - m[0][2] * a0323 + m[0][3] * a0223),
                det * -(m[0][0] * a2313 - m[0][2] * a0313 + m[0][3] * a0213),
                det *  (m[0][0] * a2312 - m[0][2] * a0312 + m[0][3] * a0212),
            ],
            [
                det *  (m[1][0] * a1323 - m[1][1] * a0323 + m[1][3] * a0123),
                det * -(m[0][0] * a1323 - m[0][1] * a0323 + m[0][3] * a0123),
                det *  (m[0][0] * a1313 - m[0][1] * a0313 + m[0][3] * a0113),
                det * -(m[0][0] * a1312 - m[0][1] * a0312 + m[0][3] * a0112),
            ],
            [
                det * -(m[1][0] * a1223 - m[1][1] * a0223 + m[1][2] * a0123),
                det *  (m[0][0] * a1223 - m[0][1] * a0223 + m[0][2] * a0123),
                det * -(m[0][0] * a1213 - m[0][1] * a0213 + m[0][2] * a0113),
                det *  (m[0][0] * a1212 - m[0][1] * a0212 + m[0][2] * a0112),
            ],
        ]
    }
}
