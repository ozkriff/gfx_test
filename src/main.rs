
#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;

use glutin::{Event, VirtualKeyCode, GlRequest};
use gfx::traits::FactoryExt;
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
    Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
    Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] }
];

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
    let pso = match window.get_api() {
        glutin::Api::OpenGl => {
            factory.create_pipeline_simple(
                include_bytes!("shader/triangle_120.glslv"),
                include_bytes!("shader/triangle_120.glslf"),
                pipe::new(),
            )
        }
        glutin::Api::OpenGlEs | glutin::Api::WebGl => {
            factory.create_pipeline_simple(
                include_bytes!("shader/triangle_100_es.glslv"),
                include_bytes!("shader/triangle_100_es.glslf"),
                pipe::new(),
            )
        }
    }.unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color
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
