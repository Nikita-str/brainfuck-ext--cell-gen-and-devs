use std::borrow::BorrowMut;

use glium::Surface;

use glutin::dpi::{Position, PhysicalPosition};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    /* 
    let width: u32 = 510;
    let height: u32 = 210;

    let event_loop = EventLoop::new();
    let win = WindowBuilder::new()
        .with_title("bf win")
        .with_decorations(false).with_transparent(true)
        .with_position(Position::Physical(PhysicalPosition::new(300, 200)))
        .with_inner_size(glutin::dpi::PhysicalSize::new(width, height));

    let ctx = ContextBuilder::new();
    let display = glium::Display::new(win, ctx, &event_loop).unwrap();

    let texture: glium::Texture2d = glium::Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        width,
        height,
    ).unwrap();

    let mut data = Vec::<u8>::with_capacity(4 * (width * height) as usize);
    for y in 0..height{
        for x in 0..width {
            if x == 0 || x == 2 || x == width - 1 || x == width - 3
            || y == 0 || y == 2 || y == height - 1 || y == height - 3
            {
                data.push(0x00);
                data.push(0x00);
                data.push(0x00);
                data.push(0xFF);
            } else if 
                (x == 1 || x == 3 || x == 4 || x == width - 2 || x == width - 4 || x == width - 5)
                ||
                (y == 1 || y == 3 || y == 4|| y == height - 2 || y == height - 4 || y == height - 5) 
            {
                data.push(0x00);
                data.push(0x00);
                data.push(0x00);
                data.push(0x00);
            }else {
                data.push(0x1F);
                data.push(0x1F);
                data.push(0x1F);
                data.push(0xFF);
            }
        }
    }

    let raw = glium::texture::RawImage2d{ 
        data: std::borrow::Cow::Borrowed( &data ),
        width,
        height,
        format: glium::texture::ClientFormat::U8U8U8U8,
    };

    texture.write(glium::Rect{left: 0, bottom: 0, width, height}, raw);

    let width = width - 10;
    let height = height - 10;
    let mut data = Vec::<u8>::with_capacity(4 * (width * height) as usize);
    for _y in 0..height{
        for _x in 0..width {
            data.push(0x7F);
            data.push(0x00);
            data.push(0x0F);
            data.push(0xFF);
        }
    }

    let display = std::sync::Arc::new(display);
    let texture = std::sync::Arc::new(texture);

    println!("H0");
 
    let frame = display.draw();
    texture
        .as_surface()
        .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
    frame.finish().unwrap();

    println!("H0.5");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let raw = glium::texture::RawImage2d{ 
        data: std::borrow::Cow::Borrowed( &data ),
        width,
        height,
        format: glium::texture::ClientFormat::U8U8U8U8,
    };

    texture.write(glium::Rect{left: 5, bottom: 5, width, height}, raw);

    let frame = display.draw();
    texture
        .as_surface()
        .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
    frame.finish().unwrap();

    println!("H1");    

    */

    bf_cell_gen::bfcg::emulators::win::Win::new(500, 200);
    

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("H2");
    }

}