//! GPU based renderer

use eframe::egui; // Use egui re-exported from eframe
use eframe::wgpu; // Use wgpu re-exported from eframe

struct GPURenderer {}

impl GPURenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl eframe::App for GPURenderer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(stt) = frame.wgpu_render_state() {
            let output = stt.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Output"),
                size: wgpu::Extent3d {
                    width: 400,
                    height: 400,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[wgpu::TextureFormat::Rgba32Float],
            });
            let view = output.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = stt
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Encoder") });

            let shader = stt.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            let render_pipeline_layout = stt.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("PipelineLayout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let render_pipeline = stt.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: stt.target_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("RenderPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&render_pipeline);
                render_pass.draw(0..3, 0..1);
            }

            stt.queue.submit(std::iter::once(encoder.finish()));
        }

        // Handle key events
        ctx.input(|stt| {
            for evt in &stt.events {
                if let egui::Event::Key {
                    key, pressed, modifiers, ..
                } = evt
                {
                    if !pressed && egui::Key::Escape == *key {
                        std::process::exit(0);
                    }
                }
            }
        });
    }
}

pub fn run(sz: (u32, u32)) -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([sz.0 as f32, sz.1 as f32]),
        ..Default::default()
    };

    let creator = Box::new(move |cc: &eframe::CreationContext<'_>| -> Box<dyn eframe::App> {
        return Box::new(GPURenderer::new());
    });

    eframe::run_native("Soft Render", options, creator)
}
