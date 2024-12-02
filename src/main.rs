use glutin_winit::GlWindow;
use winit::{
    self, application::ApplicationHandler, dpi::PhysicalSize, event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, raw_window_handle::HasWindowHandle, window::{Window, WindowId}
};
use glutin::{self, config::ConfigTemplateBuilder, context::{ContextAttributesBuilder, PossiblyCurrentContext}, display::{Display, GetGlDisplay}, prelude::{GlDisplay, NotCurrentGlContext}, surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface}};
use anyhow::Result;
use std::{f32::consts::PI, ffi::CString, num::NonZero, ops::Range, time::SystemTime};
mod helpers;
use helpers::*;
use image::{ImageBuffer, ImageFormat, Rgba};
use rand::random;

macro_rules! check_err {
    () => {
        assert_eq!(unsafe {gl::GetError() }, gl::NO_ERROR)
    };
}

#[derive(Copy, Clone)]
struct ThreeDVertex {
    position: [f32; 3],
}

struct WindowData {
    window : Window,
    _display : Display,
    surface : Surface<WindowSurface>,
    waterdrawcall : DrawCall<Vec3, WaterDrawUniform>,
    islandrawcall : DrawCall<Vec3, IslandDrawUniform>,
    treedrawcall : DrawCall<Vec2, TreeDrawUniform>,
    ctx : PossiblyCurrentContext,
}
struct App {
    windowdata : Option<WindowData>,
    draw_sea : bool,
    draw_island : bool,
    draw_trees : bool,
    start: SystemTime,
    simulation_details: SimulationDetail,
    water_tex: Rbg8Image,
    water_size : f32,
    daytime : f32,
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
            tesselation_level: NonZero::new(8).unwrap(),
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

     
    fn new(water_tex : Rbg8Image) -> Self {
        Self {
            windowdata : None,
            draw_sea : true,
            draw_island : true,
            draw_trees : true,
            water_tex,
            water_size : 4.0,
            daytime : 10.0,
            start: SystemTime::now(),
            camera: Camera::new(19.0, 0.15 * PI, 0.5 * PI, (500u32,500u32).into()),
            simulation_details: Default::default(),
        }
    }
}

struct WaterDrawUniform {
    anim_time : Uniform<f32>,
    model_view_matrix : Uniform<[[f32;4];4]>,
    aspect_ratio : Uniform<f32>,
    tess_level : Uniform<i32>,
    daytime : Uniform<f32>,
    water_size : Uniform<f32>,
    water_tex : Uniform<Texture2D>

}

fn make_water_draw_call(view : [[f32;4];4], tess_lvl : i32, water_tex : &Rbg8Image, daytime : f32, water_size : f32) -> DrawCall<Vec3, WaterDrawUniform> {
    let program = Program::new(ShaderSrc { 
        vertex_shader: include_str!("plane.vert"),
        fragment_shader: include_str!("plane.frag"),
        tessellation_control_shader: Some(include_str!("plane.tesc")),
        tessellation_evaluation_shader: Some(include_str!("plane.tese")),
        geometry_shader: None,
    }).unwrap();
    let uniforms = WaterDrawUniform {
        anim_time : Uniform::new(c"anim_time", &program, 0.0),
        model_view_matrix : Uniform::new(c"model_view_matrix", &program, view),
        aspect_ratio : Uniform::new(c"aspect_ratio", &program, 1.0),
        tess_level : Uniform::new(c"tess_level", &program, tess_lvl),
        daytime : Uniform::new(c"daytime", &program, daytime),
        water_size : Uniform::new(c"water_size", &program, water_size),
        water_tex : Uniform::new(c"water_tex",&program,Texture2D::new(water_tex, &program)),
    };
    let small_square : Vec<f32> = include!("square.in").into_iter().map(|v: ThreeDVertex| ThreeDVertex {
        position: v.position.map(|f| 0.25 * f),
    }).flat_map(|v : ThreeDVertex| v.position).collect();
    DrawCall::new(program, &small_square, c"position", uniforms).unwrap()
}

struct IslandDrawUniform {
    anim_time : Uniform<f32>,
    model_view_matrix : Uniform<[[f32;4];4]>,
    aspect_ratio : Uniform<f32>,
    tess_level : Uniform<i32>,
    daytime : Uniform<f32>,
    seed : Uniform<f32>
}

fn make_island_draw_call(view : [[f32;4];4], tess_lvl : i32, daytime : f32) -> DrawCall<Vec3, IslandDrawUniform> {
    let program = Program::new(ShaderSrc { 
        vertex_shader: include_str!("island.vert"),
        fragment_shader: include_str!("island.frag"),
        tessellation_control_shader: Some(include_str!("island.tesc")),
        tessellation_evaluation_shader: Some(include_str!("island.tese")),
        geometry_shader: None,
    }).unwrap();
    let uniforms = IslandDrawUniform {
        anim_time : Uniform::new(c"anim_time", &program, 0.0),
        model_view_matrix : Uniform::new(c"model_view_matrix", &program, view),
        aspect_ratio : Uniform::new(c"aspect_ratio", &program, 1.0),
        tess_level : Uniform::new(c"tess_level", &program, tess_lvl),
        seed : Uniform::new(c"seed", &program, 0.0),
        daytime : Uniform::new(c"daytime", &program, daytime),
    };

    let tiny_square = include!("square.in").map(|v : ThreeDVertex| ThreeDVertex {
        position: [
            (v.position[0] + 1.0) / 10.0 - 1.0,
            v.position[1],
            (v.position[2] + 1.0) / 10.0 - 1.0,
        ],
    });

    let pre_tesselated_base: Vec<_> = (0..10)
        .flat_map(|i| {
            let i = i as f32 * 0.2;
            (0..10).flat_map(move |j| {
                let j = j as f32 * 0.2;
                tiny_square.map(|v| ThreeDVertex {
                    position: [v.position[0] + i, v.position[1], v.position[2] + j],
                })
            })
        }).flat_map(|v : ThreeDVertex| v.position).collect();
    DrawCall::new(program, &pre_tesselated_base, c"position", uniforms).unwrap()
}


struct TreeDrawUniform {
    model_view_matrix : Uniform<[[f32;4];4]>,
    aspect_ratio : Uniform<f32>,
    daytime : Uniform<f32>,
    seed : Uniform<f32>
}

fn make_tree_draw_call(view : [[f32;4];4], daytime : f32) -> DrawCall<Vec2, TreeDrawUniform> {
    let program = Program::new(ShaderSrc { 
        vertex_shader: include_str!("trees.vert"),
        fragment_shader: include_str!("trees.frag"),
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: Some(include_str!("trees.geom")),
    }).unwrap();
    let uniforms = TreeDrawUniform {
        model_view_matrix : Uniform::new(c"model_view_matrix", &program, view),
        aspect_ratio : Uniform::new(c"aspect_ratio", &program, 1.0),
        seed : Uniform::new(c"seed", &program, 0.0),
        daytime : Uniform::new(c"daytime", &program, daytime),
    };

    let tree_roots : Vec<_> = (0..500).flat_map(|i| {
        let f = i as f32;
        [
                (f / 300.0).sqrt() * 1.5 * f.sin(),
                (f / 300.0).sqrt() * 1.5 * f.cos()
        ]
    }).collect();
    DrawCall::new(program, &tree_roots, c"position", uniforms).unwrap()
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("gl island").with_inner_size(PhysicalSize::new(500, 500));

        let (Some(window), config) = glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes)).build(event_loop, ConfigTemplateBuilder::new(), |mut it| {
            it.next().expect("Failed to find a suitable OpenGL configuration")
        }).expect("Failed to build display") else {panic!("no window")};
        let _display = config.display();
        let not_cur_ctx = unsafe {
            _display
                .create_context(&config, &ContextAttributesBuilder::new()
                .build(Some(window.window_handle().unwrap().as_raw())))
                .unwrap()
        };

        let surface = unsafe {
            _display.create_window_surface(&config, &window.build_surface_attributes(SurfaceAttributesBuilder::<WindowSurface>::new()).unwrap())
        }.unwrap();

        let ctx = not_cur_ctx.make_current(&surface).unwrap();


        gl::load_with(|symbol| _display.get_proc_address(&CString::new(symbol).unwrap()) as *const _);

        unsafe {gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL)}check_err!();
        unsafe {gl::Enable(gl::DEPTH_TEST)}check_err!();
        unsafe {gl::Enable(gl::CULL_FACE)}check_err!();
        unsafe {gl::Enable(gl::BLEND)}check_err!();
        unsafe {gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)}check_err!();


        window.request_redraw();
        self.windowdata = Some(WindowData { window, _display, surface, 
            waterdrawcall : make_water_draw_call(self.camera.view_matrix(), 32, &self.water_tex, self.daytime, self.water_size),
            islandrawcall : make_island_draw_call(self.camera.view_matrix(), self.simulation_details.tesselation_level.get().into(), self.daytime),
            treedrawcall : make_tree_draw_call(self.camera.view_matrix(), self.daytime),
            ctx });


    }  

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(WindowData { window, surface, waterdrawcall, islandrawcall, treedrawcall, ctx, ..  }) = self.windowdata.as_mut() else {return};
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32);}check_err!();
                self.camera.update_size(size);
                waterdrawcall.update(self.camera.aspect_ratio, |u|&mut u.aspect_ratio);
                islandrawcall.update(self.camera.aspect_ratio, |u|&mut u.aspect_ratio);
                treedrawcall.update(self.camera.aspect_ratio, |u|&mut u.aspect_ratio);
            } ,
            WindowEvent::RedrawRequested => {
                unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }; check_err!();
                unsafe  { gl::ClearColor(0.15, 0.15, 0.6, 1.0);} check_err!();
                unsafe  { gl::ClearDepth(1.0);} check_err!();


                let t = SystemTime::now()
                    .duration_since(self.start)
                    .map(|d| d.as_micros() as f32 * 0.000_001)
                    .unwrap_or(0.0);
                waterdrawcall.update(t, |u|&mut u.anim_time);
                if self.draw_sea {
                    waterdrawcall.draw();
                }
                unsafe { gl::Clear(gl::DEPTH_BUFFER_BIT); }; check_err!();
                islandrawcall.update(t, |u|&mut u.anim_time);
                if self.draw_island {
                    islandrawcall.draw();
                }
                if self.draw_trees {
                    treedrawcall.draw();
                }

                surface.swap_buffers(&ctx).unwrap();

                window.request_redraw();
            },
            WindowEvent::CursorLeft { .. } | WindowEvent::MouseInput { state : ElementState::Released, button : MouseButton::Left, .. } => {
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
                        repeat,
                        state: ElementState::Pressed,
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
                    KeyCode::KeyR if !repeat => {
                        self.simulation_details.seed = random();
                        islandrawcall.update(self.simulation_details.seed, |u| &mut u.seed);
                        treedrawcall.update(self.simulation_details.seed, |u| &mut u.seed);

                    },
                    KeyCode::KeyO => {
                        self.water_size *= 20.0 / 21.0;
                        waterdrawcall.update(self.water_size, |u| &mut u.water_size);
                    },
                    KeyCode::KeyP => {
                        self.water_size *= 21.0 / 20.0;
                        waterdrawcall.update(self.water_size, |u| &mut u.water_size);
                    },
                    KeyCode::KeyW if !repeat => println!("water size : {}", self.water_size),
                    KeyCode::KeyD => {
                        self.daytime += 0.05;
                        if self.daytime > 24.0 {
                            self.daytime -= 24.0
                        }
                        waterdrawcall.update(self.daytime, |u| &mut u.daytime);
                        islandrawcall.update(self.daytime, |u| &mut u.daytime);
                        treedrawcall.update(self.daytime, |u| &mut u.daytime);
                    },
                    KeyCode::KeyS if !repeat => {
                        self.draw_sea = !self.draw_sea
                    },
                    KeyCode::KeyG if !repeat => {
                        self.draw_island = !self.draw_island
                    },
                    KeyCode::KeyT if !repeat => {
                        self.draw_trees = !self.draw_trees
                    },

                    _ => (),
                }
                islandrawcall.update(level.get() as i32, |u|&mut u.tess_level);
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
        let Some(WindowData { waterdrawcall, islandrawcall, treedrawcall,  ..  }) = self.windowdata.as_mut() else {return};
        if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
            if self.camera.is_moving {
                self.camera.phi =
                    modular_clamp(self.camera.phi + 0.002 * x as f32, 0.0..(2.0 * PI));
                self.camera.theta = (self.camera.theta + 0.01 * y as f32).clamp(0.02 * PI, 0.5 * PI);
                waterdrawcall.update(self.camera.view_matrix(), |u| &mut u.model_view_matrix);
                islandrawcall.update(self.camera.view_matrix(), |u| &mut u.model_view_matrix);
                treedrawcall.update(self.camera.view_matrix(), |u| &mut u.model_view_matrix);

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

type Rbg8Image = ImageBuffer<Rgba<u8>, Vec<u8>>;

fn main() -> Result<()> {
    let water_src : Rbg8Image = image::load(
        std::io::Cursor::new(&include_bytes!("wateranimline.png")),
        ImageFormat::Png,
    )?
    .to_rgba8();
    //let water_src = RawImage2d::from_raw_rgba_reversed(water_src.as_raw(), water_src.dimensions());

    let event_loop = EventLoop::builder().build()?;

    //let water_tex = Texture2d::new(&display, water_src)?;
    let mut app = App::new(water_src);

    event_loop.run_app(&mut app).map_err(anyhow::Error::from)
}
