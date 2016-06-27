
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

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_Tex",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

fn load_texture<R, F>(factory: &mut F, data: &[u8]) -> ShaderResourceView<R, [f32; 4]>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::tex;
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = tex::Kind::D2(width as tex::Size, height as tex::Size, tex::AaMode::Single);
    let (_, view) = factory.create_texture_const_u8::<ColorFormat>(kind, &[&img]).unwrap();
    view
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
    let (window, mut device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let shader_header = match window.get_api() {
        Api::OpenGl => include_bytes!("shader/pre_gl.glsl").to_vec(),
        Api::OpenGlEs | Api::WebGl => include_bytes!("shader/pre_gles.glsl").to_vec(),
    };
    let mut vertex_shader = shader_header.clone();
    vertex_shader.extend_from_slice(include_bytes!("shader/v.glsl"));
    let mut fragment_shader = shader_header;
    fragment_shader.extend_from_slice(include_bytes!("shader/f.glsl"));
    let pso = factory.create_pipeline_simple(
        &vertex_shader,
        &fragment_shader,
        pipe::new(),
    ).unwrap();
    let index_data: &[u16] = &[0,  1,  2,  1,  2,  3];
    let vertex_data = &[
        Vertex { pos: [ -0.5, -0.5 ], uv: [0.0, 1.0] },
        Vertex { pos: [ -0.5,  0.5 ], uv: [0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], uv: [1.0, 1.0] },
        Vertex { pos: [  0.5,  0.5 ], uv: [1.0, 0.0] },
    ];
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(vertex_data, index_data);
    let test_texture = load_texture(&mut factory, &include_bytes!("test.png")[..]);
    let sampler = factory.create_sampler_linear();
    let data = pipe::Data {
        vbuf: vertex_buffer,
        texture: (test_texture, sampler.clone()),
        out: main_color,
    };
    loop {
        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) |
                Event::Closed => return,
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
