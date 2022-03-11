use std::any::TypeId;
use winit::{window::Window, event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use std::error::Error;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use log::{Level, LevelFilter, log};
use wgpu::{include_wgsl, VertexBufferLayout};
use wgpu::util::DeviceExt;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

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
    color: [f32; 3]
}

struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32
}

impl Model {
    pub fn new(device: &wgpu::Device) -> Model {
        let pentagram_vertices = make_pentagram_vertices();
        log!(Level::Info, "Pentagram vertices: {:?}", pentagram_vertices);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&pentagram_vertices),
            usage: wgpu::BufferUsages::VERTEX
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(PENTAGRAM_INDICES),
            usage: wgpu::BufferUsages::INDEX
        });
        let num_vertices = PENTAGRAM_INDICES.len() as u32;
        Self { vertex_buffer, index_buffer, num_vertices }
    }
}

impl Vertex {
    fn desc<'a> () -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }
}

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0]},
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0]},
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0]}
];

const PENTAGON_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];


fn penta_step(n: f32) -> f32 { (-2.0 * PI/5.0) * n - PI/2.0 }

fn make_pentagram_vertices() -> Vec<Vertex> {
    vec![
        Vertex { position: [penta_step(0.0).cos()/2.0, penta_step(0.0).sin()/2.0, 0.0], color: [0.5, 0.0, 0.5] },
        Vertex { position: [penta_step(1.0).cos()/2.0, penta_step(1.0).sin()/2.0, 0.0], color: [0.5, 0.0, 0.5] },
        Vertex { position: [penta_step(2.0).cos()/2.0, penta_step(2.0).sin()/2.0, 0.0], color: [0.5, 0.0, 0.5] },
        Vertex { position: [penta_step(3.0).cos()/2.0, penta_step(3.0).sin()/2.0, 0.0], color: [0.5, 0.0, 0.5] },
        Vertex { position: [penta_step(4.0).cos()/2.0, penta_step(4.0).sin()/2.0, 0.0], color: [0.5, 0.0, 0.5] },
        Vertex { position: [penta_step(0.5).cos(), penta_step(0.5).sin(), 0.0], color: [0.5, 0.0, 0.5]},
        Vertex { position: [penta_step(1.5).cos(), penta_step(1.5).sin(), 0.0], color: [0.5, 0.0, 0.5]},
        Vertex { position: [penta_step(2.5).cos(), penta_step(2.5).sin(), 0.0], color: [0.5, 0.0, 0.5]},
        Vertex { position: [penta_step(3.5).cos(), penta_step(3.5).sin(), 0.0], color: [0.5, 0.0, 0.5]},
        Vertex { position: [penta_step(4.5).cos(), penta_step(4.5).sin(), 0.0], color: [0.5, 0.0, 0.5]},
    ]
}

const PENTAGRAM_INDICES: &[u16] = &[
    0, 2, 1,
    0, 3, 2,
    0, 4, 3,
    0, 1, 5,
    1, 2, 6,
    2, 3, 7,
    3, 4, 8,
    4, 0, 9,
];

const PENTAGON_INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];


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
    vertices: Model,
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
    fn make_pipeline(device: &wgpu::Device, shader: &wgpu::ShaderModule, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.ok_or(GraphicsError("Creating adapter failed"))?;
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            }, None,
        ).await?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).ok_or(GraphicsError("Get preferred format failed"))?,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        let background_color = wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

        let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));
        let render_pipeline = Self::make_pipeline(&device, &shader, &config);
        let shader_alter = device.create_shader_module(&include_wgsl!("shader_alter.wgsl"));
        let render_pipeline_alter = Self::make_pipeline(&device, &shader_alter, &config);

        let render_pipelines = Flip::new(render_pipeline, render_pipeline_alter);

        let vertices = Model::new(&device);

        Ok(Self { surface, device, queue, config, size, background_color, render_pipelines, vertices })
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
                let left_color = wgpu::Color { r: 1.0, g: 0.0, b: 0.2, a: 1.0 };
                let right_color = wgpu::Color { r: 0.0, g: 1.0, b: 0.2, a: 1.0 };
                self.background_color = interpolate_color(left_color, right_color, position.x / self.size.width as f64);
                true
            }
            WindowEvent::KeyboardInput { input, .. } if input.state == ElementState::Pressed => {
                input.virtual_keycode.map_or(false, |vkey| match vkey {
                    VirtualKeyCode::Space => {
                        self.render_pipelines.flip();
                        true
                    }
                    _ => false
                })
            }
            _ => false
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
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
            render_pass.set_vertex_buffer(0, self.vertices.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.vertices.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.vertices.num_vertices, 0, 0..1);
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
        Event::WindowEvent { ref event, window_id }
        if window_id == window.id() => if !state.input(event) {
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => state.resize(*new_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
                _ => {}
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw()
        }
        _ => {}
    });
}
