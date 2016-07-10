
#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate gfx_device_gl as gfx_gl;
extern crate glutin;
extern crate image;

use std::io::Cursor;
use glutin::{Api, Event, VirtualKeyCode, GlRequest};
use gfx::traits::FactoryExt;
use gfx::handle::{RenderTargetView, DepthStencilView, ShaderResourceView};
use gfx::Device;
use gfx::tex;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type SurfaceFormat = gfx::format::R8_G8_B8_A8;
pub type FullFormat = (SurfaceFormat, gfx::format::Unorm);

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_Tex",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

fn load_texture<R, F>(factory: &mut F, data: &[u8]) -> ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = tex::Kind::D2(width as tex::Size, height as tex::Size, tex::AaMode::Single);
    let (_, view) = factory.create_texture_const_u8::<ColorFormat>(kind, &[&img]).unwrap();
    view
}

fn new_pso(
    window: &glutin::Window,
    factory: &mut gfx_gl::Factory,
) -> gfx::PipelineState<gfx_gl::Resources, pipe::Meta> {
    let shader_header = match window.get_api() {
        Api::OpenGl => include_bytes!("shader/pre_gl.glsl").to_vec(),
        Api::OpenGlEs | Api::WebGl => include_bytes!("shader/pre_gles.glsl").to_vec(),
    };
    let mut vertex_shader = shader_header.clone();
    vertex_shader.extend_from_slice(include_bytes!("shader/v.glsl"));
    let mut fragment_shader = shader_header;
    fragment_shader.extend_from_slice(include_bytes!("shader/f.glsl"));
    factory.create_pipeline_simple(
        &vertex_shader,
        &fragment_shader,
        pipe::new(),
    ).unwrap()
}

// TODO: use typedefs
// TODO: remove `#[allow(dead_code)]`
struct Visualizer {
    clear_color: [f32; 4],
    window: glutin::Window,
    device: gfx_gl::Device,
    main_color: RenderTargetView<gfx_gl::Resources, (SurfaceFormat, gfx::format::Srgb)>,
    main_depth: DepthStencilView<gfx_gl::Resources, (gfx::format::D24_S8, gfx::format::Unorm)>,
    encoder: gfx::Encoder<gfx_gl::Resources, gfx_gl::CommandBuffer>,
    pso: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    data: pipe::Data<gfx_gl::Resources>,
    is_running: bool,
    slice: gfx::Slice<gfx_gl::Resources>,

    #[allow(dead_code)]
    vertex_buffer: gfx::handle::Buffer<gfx_gl::Resources, Vertex>,

    #[allow(dead_code)]
    test_texture_view: ShaderResourceView<gfx_gl::Resources, [f32; 4]>,

    #[allow(dead_code)]
    sampler: gfx::handle::Sampler<gfx_gl::Resources>,

    #[allow(dead_code)]
    factory: gfx_gl::Factory,
}

impl Visualizer {
    fn new() -> Visualizer {
        let gl_version = GlRequest::GlThenGles {
            opengles_version: (2, 0),
            opengl_version: (2, 1),
        };
        let builder = glutin::WindowBuilder::new()
            .with_title("Triangle example".to_string())
            .with_gl(gl_version);
        let (window, device, mut factory, main_color, main_depth)
            = gfx_window_glutin::init(builder);
        let encoder = factory.create_command_buffer().into();
        let index_data: &[u16] = &[0,  1,  2,  1,  2,  3];
        let vertex_data = &[
            Vertex { pos: [ -0.5, -0.5 ], uv: [0.0, 1.0] },
            Vertex { pos: [ -0.5,  0.5 ], uv: [0.0, 0.0] },
            Vertex { pos: [  0.5, -0.5 ], uv: [1.0, 1.0] },
            Vertex { pos: [  0.5,  0.5 ], uv: [1.0, 0.0] },
        ];
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(vertex_data, index_data);
        let pso = new_pso(&window, &mut factory);
        let texture_data = &include_bytes!("test.png")[..];
        let test_texture_view = load_texture(&mut factory, texture_data);
        let sampler = factory.create_sampler_linear();
        let data = pipe::Data {
            vbuf: vertex_buffer.clone(),
            texture: (test_texture_view.clone(), sampler.clone()),
            out: main_color.clone(),
        };
        Visualizer {
            clear_color: [0.0, 0.0, 1.0, 1.0],
            window: window,
            device: device,
            factory: factory,
            main_color: main_color,
            main_depth: main_depth,
            encoder: encoder,
            pso: pso,
            vertex_buffer: vertex_buffer,
            slice: slice,
            test_texture_view: test_texture_view,
            sampler: sampler,
            data: data,
            is_running: true,
        }
    }

    fn handle_event(&mut self, event: &glutin::Event) {
        match *event {
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => {
                self.is_running = false;
            },
            Event::Closed => {
                self.is_running = false;
            },
            Event::Resized(..) => {
                gfx_window_glutin::update_views(
                    &self.window,
                    &mut self.main_color,
                    &mut self.main_depth,
                );
                self.data.out = self.main_color.clone();
            },
            Event::Touch(glutin::Touch{phase, ..}) => {
                match phase {
                    glutin::TouchPhase::Moved => {
                        println!("MOVED");
                    },
                    glutin::TouchPhase::Started => {
                        self.clear_color = [1.0, 0.0, 0.0, 1.0];
                        println!("STARTED");
                    },
                    glutin::TouchPhase::Ended => {
                        self.clear_color = [0.0, 1.0, 0.0, 1.0];
                        println!("ENDED");
                    },
                    glutin::TouchPhase::Cancelled => unimplemented!(),
                }
            },
            _ => {},
        }
    }

    fn handle_events(&mut self) {
        let events: Vec<_> = self.window.poll_events().collect();
        for event in &events {
            self.handle_event(event);
        }
    }

    fn draw(&mut self) {
        self.encoder.clear(&self.data.out, self.clear_color);
        self.encoder.draw(&self.slice, &self.pso, &self.data);
        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    fn tick(&mut self) {
        self.handle_events();
        self.draw();
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut visualizer = Visualizer::new();
    while visualizer.is_running() {
        visualizer.tick();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
