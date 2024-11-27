use anyhow::Result;
use glium::{uniforms::Uniforms, winit::{
    self,
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
}, BackfaceCullingMode, Blend, Depth, DepthTest, DrawError, DrawParameters, Frame};
use glium::{
    self,
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    program::SourceCode,
    uniform, Display, Program, Surface, VertexBuffer,
};
use rand::random;
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


#[derive(Copy, Clone)]
struct FourDVertex {
    position: [f32; 4],
}
implement_vertex!(FourDVertex, position);

struct App {
    window: Window,
    display: Display<WindowSurface>,
    plane_draw : DrawData<ThreeDVertex>,
    island_draw: DrawData<ThreeDVertex>,
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
            tesselation_level: NonZero::new(20).unwrap(),
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
        island_draw : DrawData<ThreeDVertex>,
        plane_draw : DrawData<ThreeDVertex>,
    ) -> Self {
        let window_size = window.inner_size();
        Self {
            window,
            display,
            island_draw,
            plane_draw,
            start: SystemTime::now(),
            camera: Camera::new(17.0, 0.15 * PI, 0.5 * PI, window_size),
            simulation_details: Default::default()
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
                frame.clear_color_and_depth((0.15, 0.15, 0.6, 1.0), 1.0);

                let t = SystemTime::now()
                    .duration_since(self.start)
                    .map(|d| d.as_micros() as f32 * 0.000_001)
                    .unwrap_or(0.0);

                //self.camera.phi = PI * (0.5 + t / 20.0);
                self.plane_draw.draw(
                    &mut frame,
                    &uniform! { anim_time : t , model_view_matrix : self.camera.view_matrix(), aspect_ratio : self.camera.aspect_ratio, seed : self.simulation_details.seed, tess_level : 4 * i32::from(self.simulation_details.tesselation_level.get())}
                ).unwrap();

                frame.clear_depth(1.0);

                self.island_draw.draw(
                    &mut frame,
                    &uniform! { anim_time : t , model_view_matrix : self.camera.view_matrix(), aspect_ratio : self.camera.aspect_ratio, seed : self.simulation_details.seed, tess_level :  i32::from(self.simulation_details.tesselation_level.get())}
                ).unwrap();
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
                    KeyCode::NumpadAdd => *level = level.saturating_add(1)
                    ,
                    KeyCode::NumpadSubtract => 
                        *level = NonZero::new(level.get() - 1).unwrap_or(NonZero::new(1).unwrap())
                    ,
                    KeyCode::KeyS => {
                        self.simulation_details.seed = random();
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

#[derive(Debug)]
struct DrawData<T : Copy> {
    buffer: VertexBuffer<T>,
    indices: NoIndices,
    program: Program,
    drawparam : DrawParameters<'static>
}

impl<T : Copy> DrawData<T> {
    fn draw<U: Uniforms>(&self, frame : &mut Frame, uniforms : &U) -> Result<(), DrawError> {
        frame.draw(&self.buffer, self.indices, &self.program, uniforms, &self.drawparam)
    }
}
fn main() -> Result<()> {
    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let square: [ThreeDVertex; 4] = include!("square.in");

    let plane_vertex_buffer = VertexBuffer::new(&display, &square)?;

    let plane_program = Program::new(
        &display,
        SourceCode {
            vertex_shader: include_str!("plane.vert"),
            fragment_shader: include_str!("plane.frag"),
            tessellation_control_shader: Some(include_str!("plane.tesc")),
            tessellation_evaluation_shader: Some(   include_str!("plane.tese")),
            geometry_shader: None,
        },
    )?;

    let plane_call = DrawData {
        buffer : plane_vertex_buffer,
        indices : NoIndices(PrimitiveType::Patches {
            vertices_per_patch: 4,
        }),
        program : plane_program,
        drawparam : DrawParameters { depth: Depth {
            test : DepthTest::IfLess,
            write : true,
            ..Default::default()
        },
        backface_culling : BackfaceCullingMode::CullingDisabled,
        blend : Blend::alpha_blending(),
         ..Default::default() }
    };



    let island_vertex_buffer = VertexBuffer::new(&display, &square)?;

    let island_program = Program::new(
        &display,
        SourceCode {
            vertex_shader: include_str!("shader.vert"),
            fragment_shader: include_str!("shader.frag"),
            tessellation_control_shader: Some(include_str!("shader.tesc")),
            tessellation_evaluation_shader: Some(include_str!("shader.tese")),
            geometry_shader: None,
        },
    )?;

    let island_call = DrawData {
        buffer : island_vertex_buffer,
        indices : NoIndices(PrimitiveType::Patches {
            vertices_per_patch: 4,
        }),
        program : island_program,
        drawparam : DrawParameters { depth: Depth {
            test : DepthTest::IfLess,
            write : true,
            ..Default::default()
        },
        backface_culling : BackfaceCullingMode::CullClockwise,
         ..Default::default() }
    };

    let mut app = App::new(window, display, island_call, plane_call);

    event_loop.run_app(&mut app).map_err(anyhow::Error::from)
}
