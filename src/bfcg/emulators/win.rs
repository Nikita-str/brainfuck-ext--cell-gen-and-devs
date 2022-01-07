use std::sync::Arc;

use glium::Surface;

use glutin::dpi::{Position, PhysicalPosition};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

pub struct Win{
    texture: Arc<glium::Texture2d>,
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Win{
    const STD_START_POS: (u32, u32) = (200, 300);

    const BORDER_SZ: u32 = 5;
    const BORDER_2SZ: u32 = Self::BORDER_SZ * 2;

    fn get_width(&self) -> u32 { self.width }
    fn get_height(&self) -> u32 { self.height}

    fn create_raw(data: &Vec<u8>, data_w: u32, data_h: u32) -> glium::texture::RawImage2d<u8> {
        glium::texture::RawImage2d{ 
            data: std::borrow::Cow::Borrowed( data ),
            width: data_w,
            height: data_h,
            format: glium::texture::ClientFormat::U8U8U8U8,
        }
    }

    fn draw_vec(&mut self, data: &Vec<u8>, x: u32, y: u32, data_w: u32, data_h: u32){
        let raw = Self::create_raw(data, data_w, data_h);

        self.texture.write(
            glium::Rect{left: x, bottom: y, width: data_w, height: data_h}, 
            raw
        );   
    }

    fn inner_redraw(&mut self){
        let raw = Self::create_raw(&self.data, self.width, self.height);

        self.texture.write(
            glium::Rect{
                left: Self::BORDER_SZ, 
                bottom: Self::BORDER_SZ, 
                width: self.width, 
                height: self.height
            }, 
            raw
        );   
    }

    fn init_border_draw(&mut self, width: u32, height: u32){
        let big_width = width + Self::BORDER_2SZ;
        let big_height = height + Self::BORDER_2SZ;
        let mut line_x = Vec::<u8>::with_capacity(4 * big_width  as usize);
        let mut line_y = Vec::<u8>::with_capacity(4 * big_height as usize);
        
        for _ in 0..(width + Self::BORDER_2SZ) { 
            line_x.push(0x00); 
            line_x.push(0x00); 
            line_x.push(0x00); 
            line_x.push(0xFF); 
        }
        for _ in 0..(height + Self::BORDER_2SZ) { 
            line_y.push(0x00); 
            line_y.push(0x00); 
            line_y.push(0x00); 
            line_y.push(0xFF); 
        }

        let mut data = Vec::<u8>::with_capacity(4 * (width * height) as usize);
        for _y in 0..height{
            for _x in 0..width {
                    data.push(0x1F);
                    data.push(0x1F);
                    data.push(0x1F);
                    data.push(0xFF);
            }
        }

        // Transparent pixel:
        //data.push(0x00);
        //data.push(0x00);
        //data.push(0x00);
        //data.push(0x00);
        //
        //but our screen already cleared by transparent color
    
        self.draw_vec(&line_x, 0, 0, big_width, 1);
        self.draw_vec(&line_x, 0, 2, big_width, 1);
        self.draw_vec(&line_x, 0, big_height - 3, big_width, 1);
        self.draw_vec(&line_x, 0, big_height - 1, big_width, 1);

        self.draw_vec(&line_y, 0, 0, 1, big_height);
        self.draw_vec(&line_y, 2, 0, 1, big_height);
        self.draw_vec(&line_y, big_width - 3, 0, 1, big_height);
        self.draw_vec(&line_y, big_width - 1, 0, 1, big_height);

        self.draw_vec(&data, Self::BORDER_SZ, Self::BORDER_SZ, width, height);

        self.data = data;
    }

    pub fn new(width: u32, height: u32) { Self::new_all(width, height, Self::STD_START_POS) }

    pub fn new_all(width: u32, height: u32, pos: (u32, u32)) {
        let width = width + Self::BORDER_2SZ;
        let height = height + Self::BORDER_2SZ;

        let event_loop = EventLoop::new();
        let win = WindowBuilder::new()
            .with_title("[HARDWARE]:bf:screen")
            .with_decorations(false).with_transparent(true)
            .with_position(Position::Physical(PhysicalPosition::new(pos.0 as i32, pos.1 as i32)))
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

        let width = width - Self::BORDER_2SZ;
        let height = height - Self::BORDER_2SZ;
        
        let mut ret = Self{ 
            texture: Arc::new(texture),
            data: Vec::<u8>::new(),
            width,
            height,
        };

        ret.init_border_draw(width, height);

        let texture = Arc::clone(&ret.texture);

        /*
        let width = width - 10;
        let height = height - 10;
        let mut data = Vec::<u8>::with_capacity(4 * (width * height) as usize);
        for y in 0..height{
            for x in 0..width {
                data.push(0x7F);
                data.push(0x00);
                data.push(0x00);
                data.push(0xFF);
            }
        }
        let raw = glium::texture::RawImage2d{ 
            data: std::borrow::Cow::Borrowed( &data ),
            width: width,
            height: height,
            format: glium::texture::ClientFormat::U8U8U8U8,
        };
        texture.write(glium::Rect{left: 5, bottom: 5, width: width, height: height}, raw);
        */
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            
            
            println!("H++");

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    let frame = display.draw();
                    texture
                        .as_surface()
                        .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
                    frame.finish().unwrap();
                }
                _ => (),
            }
        });
    }
}