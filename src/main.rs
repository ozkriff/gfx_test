
#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
extern crate image;

use std::io::Cursor;
use glutin::{Api, Event, VirtualKeyCode, GlRequest};
use gfx::traits::FactoryExt;
use gfx::handle::{ShaderResourceView};
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
    factory: &mut gfx_device_gl::Factory,
) -> gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta> {
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

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut clear_color = [0.0, 0.0, 1.0, 1.0];
    let gl_version = GlRequest::GlThenGles {
        opengles_version: (2, 0),
        opengl_version: (2, 1),
    };
    let builder = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string())
        .with_gl(gl_version);
    let (window, mut device, mut factory, mut main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = new_pso(&window, &mut factory);
    let index_data: &[u16] = &[0,  1,  2,  1,  2,  3];
    let vertex_data = &[
        Vertex { pos: [ -0.5, -0.5 ], uv: [0.0, 1.0] },
        Vertex { pos: [ -0.5,  0.5 ], uv: [0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], uv: [1.0, 1.0] },
        Vertex { pos: [  0.5,  0.5 ], uv: [1.0, 0.0] },
    ];
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(vertex_data, index_data);
    let test_texture_view = load_texture(&mut factory, &include_bytes!("test.png")[..]);
    let sampler = factory.create_sampler_linear();
    let mut data = pipe::Data {
        vbuf: vertex_buffer.clone(),
        texture: (test_texture_view.clone(), sampler.clone()),
        out: main_color.clone(),
    };
    loop {
        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return,
                Event::Closed => return,
                Event::Resized(..) => {
                    gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                    data.out = main_color.clone();
                },
                Event::Touch(glutin::Touch{phase, ..}) => {
                    match phase {
                        glutin::TouchPhase::Moved => {
                            println!("MOVED");
                        },
                        glutin::TouchPhase::Started => {
                            clear_color = [1.0, 0.0, 0.0, 1.0];
                            println!("STARTED");
                        },
                        glutin::TouchPhase::Ended => {
                            clear_color = [0.0, 1.0, 0.0, 1.0];
                            println!("ENDED");
                        },
                        glutin::TouchPhase::Cancelled => unimplemented!(),
                    }
                },
                _ => {},
            }
        }
        encoder.clear(&data.out, clear_color);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
