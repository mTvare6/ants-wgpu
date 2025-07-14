use glam::Vec2;
use image::ImageReader;
use rand::{rngs::ThreadRng, Rng};
use std::sync::Arc;
use std::{f32::consts::PI, iter};
use wgpu::util::DeviceExt;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const SCREEN_WIDTH: u32 = 1600;
const SCREEN_HEIGHT: u32 = 1000;
const NUM_ANTS: u32 = 4096;
const AWAY: u32 = 0;
const HOME: Vec2 = Vec2::new(1128., 782.);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Ant {
    pos: [f32; 2],
    angle: f32,
    state: u32,
}

impl Ant {
    fn new(rng: &mut ThreadRng) -> Self {
        let angle = rng.gen_range(0.0..PI * 2.);
        let vec = Vec2::from_angle(angle);
        let state = AWAY;
        Ant {
            pos: (HOME + vec * 30.).into(),
            angle,
            state,
        }
    }
}

fn create_world_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("World Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let img = ImageReader::open("src/world.png")
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");

    let rgba_img = img.to_rgba8();
    assert_eq!((width, height), rgba_img.dimensions());
    let world_data = rgba_img.into_raw();

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &world_data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Ants")
            .with_inner_size(LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
            .build(&event_loop)
            .unwrap(),
    );

    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0];
    let config = surface
        .get_default_config(&adapter, SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap();
    surface.configure(&device, &config);

    let mut rng = rand::thread_rng();
    let ants: Vec<Ant> = (0..NUM_ANTS).map(|_| Ant::new(&mut rng)).collect();

    let ant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Ant Buffer"),
        contents: bytemuck::cast_slice(&ants),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let world_textures = [
        create_world_texture(&device, &queue, SCREEN_WIDTH, SCREEN_HEIGHT),
        create_world_texture(&device, &queue, SCREEN_WIDTH, SCREEN_HEIGHT),
    ];

    let world_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let ant_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Ant Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
    });

    let process_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Process Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("process.wgsl").into()),
    });

    let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("render.wgsl").into()),
    });

    let ant_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Ant Compute Pipeline"),
        layout: None,
        module: &ant_compute_shader,
        entry_point: "main",
    });

    let process_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Process Pipeline"),
        layout: None,
        module: &process_shader,
        entry_point: "main",
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &render_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &render_shader,
            entry_point: "fs_main",
            targets: &[Some(surface_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let ant_bind_group_layout = ant_compute_pipeline.get_bind_group_layout(0);
    let process_bind_group_layout = process_pipeline.get_bind_group_layout(0);
    let render_bind_group_layout = render_pipeline.get_bind_group_layout(0);

    let ant_bind_groups = [
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ant Bind Group 0"),
            layout: &ant_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ant_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&world_textures[0]),
                },
            ],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ant Bind Group 1"),
            layout: &ant_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ant_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&world_textures[1]),
                },
            ],
        }),
    ];

    let process_bind_groups = [
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Process Bind Group 0"),
            layout: &process_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&world_textures[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[1]),
                },
            ],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Process Bind Group 1"),
            layout: &process_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&world_textures[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[0]),
                },
            ],
        }),
    ];

    let render_bind_groups = [
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group 0"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&world_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[0]),
                },
            ],
        }),
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group 1"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&world_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&world_textures[1]),
                },
            ],
        }),
    ];

    let mut frame_num = 0;
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    let frame = surface.get_current_texture().unwrap();
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                    let idx = frame_num % 2;

                    {
                        let mut compute_pass =
                            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Process Pass"),
                                timestamp_writes: None,
                            });
                        compute_pass.set_pipeline(&process_pipeline);
                        compute_pass.set_bind_group(0, &process_bind_groups[idx], &[]);
                        compute_pass.dispatch_workgroups(SCREEN_WIDTH / 8, SCREEN_HEIGHT / 8, 1);
                    }

                    {
                        let mut compute_pass =
                            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Ant Pass"),
                                timestamp_writes: None,
                            });
                        compute_pass.set_pipeline(&ant_compute_pipeline);
                        compute_pass.set_bind_group(0, &ant_bind_groups[idx], &[]);
                        compute_pass.dispatch_workgroups(NUM_ANTS / 64, 1, 1);
                    }

                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                        render_pass.set_pipeline(&render_pipeline);
                        render_pass.set_bind_group(0, &render_bind_groups[idx], &[]);
                        render_pass.draw(0..3, 0..1);
                    }

                    queue.submit(iter::once(encoder.finish()));
                    frame.present();
                    frame_num += 1;
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
