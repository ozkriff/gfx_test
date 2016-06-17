
#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;

use glutin::{Event, VirtualKeyCode};
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    pipeline pipe {
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let clear_color = [0.1, 0.2, 0.3, 1.0];
    let gl_version = glutin::GlRequest::GlThenGles {
        opengles_version: (2, 0),
        opengl_version: (2, 1),
    };
    let builder = glutin::WindowBuilder::new()
        .with_title("Basic example".to_string())
        .with_gl(gl_version);
    let (window, mut device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let data = pipe::Data{out: main_color};
    loop {
        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) | Event::Closed => {
                    return;
                }
                _ => {}
            }
        }
        encoder.clear(&data.out, clear_color);
        encoder.flush(&mut device);
        window.swap_buffers().expect("TEST TEST TEST");
        device.cleanup();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
