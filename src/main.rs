use crate::texture::Texture;
use anyhow::{Context, Result};
use image::ImageFormat::Png;
use image::GenericImageView;
use log::{log, Level, LevelFilter};
use serde::Deserialize;
use std::any::TypeId;
use std::error::Error;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use std::fs::File;
use wgpu::util::DeviceExt;
use wgpu::{include_wgsl, VertexBufferLayout};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

mod texture;

#[derive(Debug)]
struct GraphicsError(&'static str);

impl Display for GraphicsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for GraphicsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        self.0
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Deserialize)]
struct ModelData {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u16>,
}

impl ModelData {
    pub fn vertices(&self) -> Vec<Vertex> {
        self.positions
            .iter()
            .zip(self.uvs.iter())
            .map(|(&position, &uv)| Vertex { position, uv })
            .collect()
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}

struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl Model {
    fn load() -> ModelData {
        serde_json::from_str(include_str!("rectangle.model")).expect("model data layout")
    }
    pub fn new(device: &wgpu::Device) -> Model {
        let model_data = Self::load();
        let vertices = model_data.vertices();
        log!(Level::Info, "vertices = #{:?}", vertices);
        let indices = model_data.indices();
        log!(Level::Info, "indices = #{:?}", indices);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_vertices = indices.len() as u32;
        Self {
            vertex_buffer,
            index_buffer,
            num_vertices,
        }
    }
}

struct Flip<T> {
    alternatives: [T; 2],
    state: bool,
}

impl<T> Flip<T> {
    pub fn new(first: T, second: T) -> Flip<T> {
        Flip {
            alternatives: [first, second],
            state: false,
        }
    }

    pub fn flip(&mut self) {
        self.state = !self.state;
    }

    pub fn get(&self) -> &T {
        &self.alternatives[if self.state { 1 } else { 0 }]
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    background_color: wgpu::Color,
    render_pipelines: Flip<wgpu::RenderPipeline>,
    model: Model,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,
}

fn interpolate_color(from: wgpu::Color, to: wgpu::Color, factor: f64) -> wgpu::Color {
    wgpu::Color {
        r: from.r + (to.r - from.r) * factor,
        g: from.g + (to.g - from.g) * factor,
        b: from.b + (to.b - from.b) * factor,
        a: from.a + (to.a - from.a) * factor,
    }
}

impl State {
    fn make_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        config: &wgpu::SurfaceConfiguration,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
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
        })
    }

    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GraphicsError("Creating adapter failed"))?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("Device"),
                },
                None,
            )
            .await?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_preferred_format(&adapter)
                .ok_or(GraphicsError("Get preferred format failed"))?,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let image = image::load(
            std::io::BufReader::new(
                File::open("assets/tree.png").context("failed to open assets/tree.png")?,
            ),
            Png,
        )
        .context("failed to read tree as PNG")?;
        let diffuse_texture =
            texture::Texture::from_image(&device, &queue, &image, Some("tree.png"))?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(diffuse_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(diffuse_texture.sampler()),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let background_color = wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));
        let render_pipeline =
            Self::make_pipeline(&device, &shader, &config, &texture_bind_group_layout);
        let shader_alter = device.create_shader_module(&include_wgsl!("shader_alter.wgsl"));
        let render_pipeline_alter =
            Self::make_pipeline(&device, &shader_alter, &config, &texture_bind_group_layout);

        let render_pipelines = Flip::new(render_pipeline, render_pipeline_alter);

        let model = Model::new(&device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            background_color,
            render_pipelines,
            model,
            diffuse_bind_group,
            diffuse_texture,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("Resizing to {:?}", new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let left_color = wgpu::Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.2,
                    a: 1.0,
                };
                let right_color = wgpu::Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.2,
                    a: 1.0,
                };
                self.background_color =
                    interpolate_color(left_color, right_color, position.x / self.size.width as f64);
                true
            }
            WindowEvent::KeyboardInput { input, .. } if input.state == ElementState::Pressed => {
                input.virtual_keycode.map_or(false, |vkey| match vkey {
                    VirtualKeyCode::Space => {
                        self.render_pipelines.flip();
                        true
                    }
                    _ => false,
                })
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipelines.get());
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.model.num_vertices, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let mut state = pollster::block_on(State::new(&window))?;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => state.resize(*new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => window.request_redraw(),
        _ => {}
    });
}
