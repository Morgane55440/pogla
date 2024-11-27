use anyhow::Result;
use glium::winit::{
    self,
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};
use glium::{
    self,
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    program::SourceCode,
    uniform, Display, Program, Surface, VertexBuffer,
};
use std::{f32::consts::PI, num::NonZero, ops::Range, time::SystemTime};

#[derive(Copy, Clone)]
struct TwoDVertex {
    position: [f32; 2],
}
implement_vertex!(TwoDVertex, position);

#[derive(Copy, Clone)]
struct ThreeDVertex {
    position: [f32; 3],
}
implement_vertex!(ThreeDVertex, position);

struct App {
    window: Window,
    display: Display<WindowSurface>,
    buffer: VertexBuffer<ThreeDVertex>,
    indices: NoIndices,
    program: Program,
    start: SystemTime,
    simulation_details: SimulationDetail,
    camera: Camera,
}
#[derive(Copy, Clone, Debug)]
pub struct SimulationDetail {
    pub tesselation_level: NonZero<u8>,
    pub seed: f32,
}

impl Default for SimulationDetail {
    fn default() -> Self {
        Self {
            tesselation_level: NonZero::new(100).unwrap(),
            seed: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub distance: f32,
    pub theta: f32,
    pub phi: f32,
    pub is_moving: bool,
    pub aspect_ratio: f32,
}

impl Camera {
    fn new(distance: f32, theta: f32, phi: f32, windowsize: PhysicalSize<u32>) -> Self {
        let aspect_ratio = windowsize.width as f32 / windowsize.height as f32;
        Self {
            distance,
            theta,
            phi,
            is_moving: false,
            aspect_ratio,
        }
    }

    fn update_size(&mut self, windowsize: PhysicalSize<u32>) {
        self.aspect_ratio = windowsize.width as f32 / windowsize.height as f32;
    }
    fn view_matrix(self) -> [[f32; 4]; 4] {
        let (cos1, cos2, sin1, sin2) = (
            self.phi.cos(),
            self.theta.cos(),
            self.phi.sin(),
            self.theta.sin(),
        );
        [
            [cos1, sin1 * sin2, -sin1 * cos2, 0.00000],
            [0.00000, cos2, sin2, 0.00000],
            [sin1, -sin2 * cos1, cos1 * cos2, 0.00000],
            [0.00000, 0.00000, -self.distance, 1.00000],
        ]
    }
}

impl App {
    fn new(
        window: Window,
        display: Display<WindowSurface>,
        buffer: VertexBuffer<ThreeDVertex>,
        indices: NoIndices,
        program: Program,
    ) -> Self {
        let window_size = window.inner_size();
        Self {
            window,
            display,
            buffer,
            indices,
            program,
            start: SystemTime::now(),
            camera: Camera::new(15.0, 0.15 * PI, 0.5 * PI, window_size),
            simulation_details: Default::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.display.resize(size.into());
                self.camera.update_size(size);
            }
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.draw();
                frame.clear_color(0.4, 0.4, 0.4, 1.0);

                let t = SystemTime::now()
                    .duration_since(self.start)
                    .map(|d| d.as_micros() as f32 * 0.000_001)
                    .unwrap_or(0.0);

                //self.camera.phi = PI * (0.5 + t / 20.0);

                frame
                    .draw(
                        &self.buffer,
                        self.indices,
                        &self.program,
                        &uniform! { anim_time : t , model_view_matrix : self.camera.view_matrix(), aspect_ratio : self.camera.aspect_ratio, seed : self.simulation_details.seed, tess_level : i32::from(self.simulation_details.tesselation_level.get())},
                        &Default::default(),
                    )
                    .unwrap();
                frame.finish().unwrap();
                self.window.request_redraw();
            }
            WindowEvent::CursorLeft { .. }
            | WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.camera.is_moving = false;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.camera.is_moving = true;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let level = &mut self.simulation_details.tesselation_level;
                match keycode {
                    KeyCode::NumpadAdd => *level = level.saturating_add(1),
                    KeyCode::NumpadSubtract => {
                        *level = NonZero::new(level.get() - 1).unwrap_or(NonZero::new(1).unwrap())
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
            if self.camera.is_moving {
                self.camera.phi =
                    modular_clamp(self.camera.phi + 0.002 * x as f32, 0.0..(2.0 * PI));
                self.camera.theta = (self.camera.theta + 0.01 * y as f32).clamp(0.02 * PI, 0.5 * PI)
            }
        }
    }
}

fn modular_clamp(mut x: f32, range: Range<f32>) -> f32 {
    while x < range.start {
        x += range.end - range.start;
    }
    while x > range.end {
        x -= range.end - range.start;
    }
    x
}
fn main() -> Result<()> {
    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let square: [ThreeDVertex; 4] = include!("square.in");
    let vertex_buffer = VertexBuffer::new(&display, &square)?;
    let indices = NoIndices(PrimitiveType::Patches {
        vertices_per_patch: 4,
    });

    let program = Program::new(
        &display,
        SourceCode {
            vertex_shader: include_str!("shader.vert"),
            fragment_shader: include_str!("shader.frag"),
            tessellation_control_shader: Some(include_str!("shader.tesc")),
            tessellation_evaluation_shader: Some(include_str!("shader.tese")),
            geometry_shader: None,
        },
    )?;

    let mut app = App::new(window, display, vertex_buffer, indices, program);

    event_loop.run_app(&mut app).map_err(anyhow::Error::from)
}
