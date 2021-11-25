use std::{error::Error, fmt};

use cgmath::{num_traits::Pow, prelude::*};
use color_eyre::eyre::Context;
use futures::executor::block_on;
use sdl2::event::Event;
use wgpu::util::DeviceExt;

mod camera;
mod particle;
mod particle_trail;

fn string_err(s: String) -> StrErr {
    StrErr { s }
}

#[derive(Debug)]
struct StrErr {
    s: String,
}

impl fmt::Display for StrErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}
impl Error for StrErr {}

fn create_a_damn_circle(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let mut vertexes: Vec<f32> = vec![0.0, 0.0, 1.0, 0.0];
    let mut indexes: Vec<u16> = vec![];
    const EDGE_VERTEX_COUNT: u16 = 100;
    for index in 1..=EDGE_VERTEX_COUNT {
        let theta = index as f32 / EDGE_VERTEX_COUNT as f32 * std::f32::consts::PI * 2.0;
        let x = theta.cos();
        let y = -theta.sin(); // negative to make it counter-clockwise
        vertexes.push(x);
        vertexes.push(y);
        indexes.push(0);
        indexes.push(index);
        indexes.push(index + 1);
    }
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("circle vertex buffer"),
        contents: bytemuck::cast_slice(&vertexes[..]),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("circle index buffer"),
        contents: bytemuck::cast_slice(&indexes[..]),
        usage: wgpu::BufferUsages::INDEX,
    });
    (vertex_buffer, index_buffer, 3 * EDGE_VERTEX_COUNT as u32)
}

fn and_a_square_too(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("square vertex buffer"),
        contents: bytemuck::cast_slice(&[
            -1.0f32, -1.0f32, 1.0f32, 1.0f32, 1.0f32, -1.0f32, -1.0f32, 1.0f32,
        ]),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("square index buffer"),
        contents: bytemuck::cast_slice::<u16, _>(&[0, 1, 2, 0, 3, 1]),
        usage: wgpu::BufferUsages::INDEX,
    });
    (vertex_buffer, index_buffer, 6)
}

fn correct_pos(
    (x, y): (f32, f32),
    config: &wgpu::SurfaceConfiguration,
    projection: cgmath::Matrix4<f32>,
) -> cgmath::Point2<f32> {
    let p = cgmath::point3(
        x / config.width as f32 * 2.0,
        y / config.height as f32 * 1.6,
        0.0,
    );
    let p = projection.inverse_transform().unwrap().transform_point(p);
    cgmath::point2(p.x, p.y)
}
fn correct_rel(
    (x, y): (f32, f32),
    config: &wgpu::SurfaceConfiguration,
    projection: cgmath::Matrix4<f32>,
) -> cgmath::Vector2<f32> {
    let v = cgmath::vec3(
        x / config.width as f32 * 2.0,
        y / config.height as f32 * 1.6,
        0.0,
    );
    let v = projection.inverse_transform_vector(v).unwrap();
    cgmath::vec2(v.x, v.y)
}

fn main() -> color_eyre::Result<()> {
    env_logger::init();
    let sdl_context = sdl2::init().map_err(string_err)?;
    let video = sdl_context.video().map_err(string_err)?;
    let window = video
        .window("Atomica", 800, 600)
        .position_centered()
        .build()?;
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .ok_or_else(|| string_err("unable to find adapter".into()))?;
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("GPU"),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))?;

    let preferred_format = surface
        .get_preferred_format(&adapter)
        .ok_or_else(|| string_err("no preferred format".into()))?;

    let main_circle_vert =
        device.create_shader_module(&wgpu::include_spirv!("main_circle.vert.spirv"));
    let main_circle_frag =
        device.create_shader_module(&wgpu::include_spirv!("main_circle.frag.spirv"));
    let trail_vert = device.create_shader_module(&wgpu::include_spirv!("trail.vert.spirv"));
    let trail_frag = device.create_shader_module(&wgpu::include_spirv!("trail.frag.spirv"));
    //insert shader stuff here

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("transform uniform layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let common_primitive = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        clamp_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
    };
    let common_targets = [wgpu::ColorTargetState {
        format: preferred_format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }];

    let main_circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main circle pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &main_circle_vert,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<particle::RawParticle>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &main_circle_frag,
            entry_point: "main",
            targets: &common_targets,
        }),
        primitive: common_primitive,
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    let trail_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("trail pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &trail_vert,
            entry_point: "main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<particle_trail::RawTrail>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32, 4 => Float32],
                },
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &trail_frag,
            entry_point: "main",
            targets: &common_targets
        }),
        primitive: common_primitive,
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    let (width, height) = window.size();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: preferred_format,
        width,
        height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    let (circle_vertexes, circle_indexes, circle_index_count) = create_a_damn_circle(&device);
    let (square_vertexes, square_indexes, square_index_count) = and_a_square_too(&device);

    let mut particles = vec![
        particle::Particle::new(
            cgmath::Point2::new(-6.0, 0.75),
            cgmath::Vector2::new(1.0, 0.0),
            1.0,
            1.0,
        ),
        particle::Particle::new(
            cgmath::Point2::new(6.0, -0.75),
            cgmath::Vector2::new(-1.0, 0.0),
            1.0,
            -1.0,
        ),
    ];

    let mut trails = particle_trail::TrailManager::new();

    #[rustfmt::skip]
    let opengl_to_wgpu = cgmath::Matrix4::<f32>::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    );

    let mut projection_matrix = opengl_to_wgpu
        * cgmath::ortho(
            -1.0,
            1.0,
            1.0 * (height as f32 / width as f32),
            -1.0 * (height as f32 / width as f32),
            -1.0,
            1.0,
        );

    let mut camera = camera::Camera::new();

    let mut accumulated_time = std::time::Duration::ZERO;
    let mut last_frame = std::time::Instant::now();

    let mut sdl_pump = sdl_context.event_pump().map_err(string_err)?;
    'game_loop: loop {
        for event in sdl_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    break 'game_loop;
                }
                Event::Window {
                    window_id,
                    win_event: sdl2::event::WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == window.id() => {
                    surface_config.width = width as u32;
                    surface_config.height = height as u32;
                    surface.configure(&device, &surface_config);
                    projection_matrix = opengl_to_wgpu
                        * cgmath::ortho(
                            -1.0,
                            1.0,
                            1.0 * (surface_config.height as f32 / surface_config.width as f32),
                            -1.0 * (surface_config.height as f32 / surface_config.width as f32),
                            -1.0,
                            1.0,
                        );
                }
                Event::MouseButtonDown { which: 0, x, y, .. } => camera.click_mouse(correct_pos(
                    (x as _, -y as _),
                    &surface_config,
                    projection_matrix,
                )),
                Event::MouseMotion {
                    which: 0,
                    x,
                    y,
                    xrel,
                    yrel,
                    ..
                } => camera.drag_mouse(
                    correct_pos((x as _, -y as _), &surface_config, projection_matrix),
                    correct_rel((xrel as _, -yrel as _), &surface_config, projection_matrix),
                ),
                Event::MouseButtonUp { which: 0, .. } => {
                    camera.let_go_of_mouse();
                }
                Event::MouseWheel {
                    mut y, direction, ..
                } => {
                    y *= direction.to_ll() as i32 * 2 - 1;
                    camera.scroll(y as f32 / 20.0);
                }
                _ => {}
            }
        }

        let update_time: std::time::Duration = std::time::Duration::new(0, 16666667);
        while accumulated_time > update_time {
            //yay, the update loop
            particle::Particle::update(&mut particles, update_time.as_secs_f64());
            trails.update(update_time, &particles);
            accumulated_time -= update_time
        }

        let particle_raws = particles.iter().map(|p| p.to_raw()).collect::<Vec<_>>();
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle buffer"),
            contents: bytemuck::cast_slice(&particle_raws[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let trail_buffer = trails.get_buffer(&device);

        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform buffer"),
            contents: bytemuck::cast_slice(&cgmath::conv::array4x4(
                projection_matrix * camera.create_matrix(),
            )),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform uniform buffer"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    transform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let frame = surface
            .get_current_texture()
            .context("failed to get next frame from surface")?;
        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.102f64.pow(2.0),
                            g: 0.090f64.pow(2.0),
                            b: 0.098f64.pow(2.0),
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&trail_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_vertex_buffer(0, square_vertexes.slice(..));
            rpass.set_vertex_buffer(1, trail_buffer.slice(..));
            rpass.set_index_buffer(square_indexes.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..square_index_count, 0, 0..trails.len());
            rpass.set_pipeline(&main_circle_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_vertex_buffer(0, circle_vertexes.slice(..));
            rpass.set_vertex_buffer(1, particle_buffer.slice(..));
            rpass.set_index_buffer(circle_indexes.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..circle_index_count, 0, 0..particles.len() as u32);
        }
        queue.submit([encoder.finish()]);
        frame.present();

        let new_time = std::time::Instant::now();
        accumulated_time += new_time - last_frame;
        last_frame = new_time;
    }
    println!("Hello, world!");
    Ok(())
}
