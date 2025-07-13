use glam::Vec2;
use rand::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Ant {
    pos: [f32; 2],
    vel: [f32; 2],
}

const ANTS_COUNT: usize = 100;

fn create_pheromone_texture(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Pheromone Texture"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Ants Simulation")
            .build(&event_loop)
            .unwrap(),
    );

    let mut size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let mut rng = thread_rng();

    // ants gen


    let ants: Vec<Ant> = (0..ANTS_COUNT)
        .map(|_| {
            let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
            let vec = Vec2::from_angle(angle);
            Ant {
                pos: [
                    vec.x * 50. + size.width as f32 / 2.,
                    vec.y * 50. + size.height as f32 / 2.,
                ],
                vel: [vec.x, vec.y],
            }
        })
        .collect();

    let ant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Ant Buffer"),
        contents: bytemuck::cast_slice(&ants),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST,
    });

    let mut pheromone_a = create_pheromone_texture(&device, size);
    let mut pheromone_b = create_pheromone_texture(&device, size);

    let ant_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Ant Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
    });
    let decay_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Decay Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("decay.wgsl"))),
    });
    let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("render.wgsl"))),
    });

    let ant_compute_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ant Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

    let ant_compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ant Compute Pipeline Layout"),
            bind_group_layouts: &[&ant_compute_bind_group_layout],
            push_constant_ranges: &[],
        });

    let ant_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Ant Compute Pipeline"),
        layout: Some(&ant_compute_pipeline_layout),
        module: &ant_compute_shader,
        entry_point: "main",
    });

    let decay_compute_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Decay Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

    let decay_compute_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Decay Compute Pipeline Layout"),
            bind_group_layouts: &[&decay_compute_bind_group_layout],
            push_constant_ranges: &[],
        });

    let decay_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Decay Compute Pipeline"),
        layout: Some(&decay_compute_pipeline_layout),
        module: &decay_compute_shader,
        entry_point: "main",
    });

    let render_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
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

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&render_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pheromone_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Pheromone Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &render_shader,
            entry_point: "vs_pheromone",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &render_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let ant_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Ant Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &render_shader,
            entry_point: "vs_ant",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Ant>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &render_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut frame_num = 0;

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } if window_id == window.id() => elwt.exit(),
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(new_size),
                ..
            } if window_id == window.id() => {
                size = new_size;
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                pheromone_a = create_pheromone_texture(&device, size);
                pheromone_b = create_pheromone_texture(&device, size);
                window.request_redraw();
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
                ..
            } if window_id == window.id() => {
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        surface.configure(&device, &config);
                        return;
                    }
                    Err(e) => {
                        eprintln!("Failed to get current texture: {:?}", e);
                        return;
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let (src_pheromone, dst_pheromone) = if frame_num % 2 == 0 {
                    (&pheromone_a, &pheromone_b)
                } else {
                    (&pheromone_b, &pheromone_a)
                };
                let src_view = src_pheromone.create_view(&wgpu::TextureViewDescriptor::default());
                let dst_view = dst_pheromone.create_view(&wgpu::TextureViewDescriptor::default());

                {
                    let decay_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Decay Bind Group"),
                        layout: &decay_compute_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&src_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&dst_view),
                            },
                        ],
                    });
                    let mut compute_pass =
                        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Decay Pass"),
                            timestamp_writes: None,
                        });
                    compute_pass.set_pipeline(&decay_compute_pipeline);
                    compute_pass.set_bind_group(0, &decay_bind_group, &[]);
                    compute_pass.dispatch_workgroups(size.width / 8, size.height / 8, 1);
                }

                {
                    let ant_compute_bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Ant Compute Bind Group"),
                            layout: &ant_compute_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: ant_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&dst_view),
                                },
                            ],
                        });
                    let mut compute_pass =
                        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Ant Compute Pass"),
                            timestamp_writes: None,
                        });
                    compute_pass.set_pipeline(&ant_compute_pipeline);
                    compute_pass.set_bind_group(0, &ant_compute_bind_group, &[]);
                    compute_pass.dispatch_workgroups(
                        (ANTS_COUNT as f32 / 64.0).ceil() as u32,
                        1,
                        1,
                    );
                }

                {
                    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Render Bind Group"),
                        layout: &render_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&dst_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                        ],
                    });
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(&pheromone_pipeline);
                    render_pass.set_bind_group(0, &render_bind_group, &[]);
                    render_pass.draw(0..6, 0..1);

                    render_pass.set_pipeline(&ant_pipeline);
                    render_pass.set_bind_group(0, &render_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, ant_buffer.slice(..));
                    render_pass.draw(0..6, 0..ANTS_COUNT as u32);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();

                frame_num += 1;
            }
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
