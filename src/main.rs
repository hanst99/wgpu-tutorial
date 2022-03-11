use std::any::TypeId;
use winit::{window::Window, event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use std::error::Error;
use std::fmt::{Display, Formatter};
use wgpu::include_wgsl;

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
                buffers: &[],
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

        Ok(Self { surface, device, queue, config, size, background_color, render_pipelines })
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
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::init();
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
