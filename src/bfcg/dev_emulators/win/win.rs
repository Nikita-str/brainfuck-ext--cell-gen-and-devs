use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use glium::Surface;

use glutin::dpi::{Position, PhysicalPosition};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::window::{WindowBuilder, WindowId};
use glutin::ContextBuilder;

pub struct Win{
    need_redraw: Arc<AtomicBool>,
    data: Arc<Mutex<Vec<u8>>>,
    width: u32,
    height: u32,
    drawer: Option<DrawThreadWin>,
}

struct DrawThreadWin{
    can_thread_draw: bool,
    proxy: EventLoopProxy<Event<'static, ()>>,
    id: WindowId,
}

#[derive(Clone, Copy)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl std::clone::Clone for Win{
    fn clone(&self) -> Self {
        Self { 
            need_redraw: Arc::clone(&self.need_redraw), 
            data: Arc::clone(&self.data), 
            width: self.width, 
            height: self.height, 
            drawer: None,
        }
    }
}

impl Win{
    const STD_START_POS: (u32, u32) = (200, 300);

    const WAIT_MS: u64 = 10;

    const BORDER_SZ: u32 = 5;
    const BORDER_2SZ: u32 = Self::BORDER_SZ * 2;

    
    fn get_mut_drawer(&mut self) -> &mut DrawThreadWin { self.drawer.as_mut().unwrap() }
    fn with_drawer(&self) -> bool{ self.drawer.is_some() }
    fn can_thread_draw(&self) -> bool{ self.with_drawer() && self.drawer.as_ref().unwrap().can_thread_draw }

    pub fn get_width(&self) -> u32 { self.width }
    pub fn get_height(&self) -> u32 { self.height }

    pub fn start_draw_frame(&mut self) -> bool{
        if !self.with_drawer() { return false }
        while self.need_redraw.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(Self::WAIT_MS));
        }
        self.get_mut_drawer().can_thread_draw = true;
        true
    }

    pub fn end_draw_frame(&mut self) -> bool{
        if !self.with_drawer() { return false }
        
        self.get_mut_drawer().can_thread_draw = false;
        self.need_redraw.store(true, Ordering::Relaxed); 
        
        let drawer = self.get_mut_drawer();    
        let event = Event::RedrawRequested(drawer.id);
        if drawer.proxy.send_event(event).is_err() {
            false
        } else {
            true
        } 
    }

    pub fn clear_color(&mut self, color: Color){
        // for test only

        if !self.can_thread_draw() {
            panic!("cant draw"); // TODO: return as bad result (false)
        }
        let mut data = self.data.lock().unwrap();
        for y in 0..self.height{
            for x in 0..self.width {
                    let ptr = 4 * (y * self.width + x) as usize;
                    data[ptr + 0] = color.r;
                    data[ptr + 1] = color.g;
                    data[ptr + 2] = color.b;
                    data[ptr + 3] = color.a;
            }
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color){
        let mut data = self.data.lock().unwrap();
        let ptr = 4 * (y * self.width + x) as usize;
        if color.a == 0 { 
            data[ptr + 0] = 0;
            data[ptr + 1] = 0;
            data[ptr + 2] = 0;
            data[ptr + 3] = 0;
        } else { 
            data[ptr + 0] = color.r;
            data[ptr + 1] = color.g;
            data[ptr + 2] = color.b;
            data[ptr + 3] = 0xFF;
        };
    }

    pub fn get_pixel(&mut self, x: u32, y: u32) -> Color {
        let data = self.data.lock().unwrap();
        let ptr = 4 * (y * self.width + x) as usize;
        return Color{ r: data[ptr + 0], g: data[ptr + 1], b: data[ptr + 2], a: data[ptr + 3] }
    }


    fn create_raw(data: &Vec<u8>, data_w: u32, data_h: u32) -> glium::texture::RawImage2d<u8> {
        glium::texture::RawImage2d{ 
            data: std::borrow::Cow::Borrowed( data ),
            width: data_w,
            height: data_h,
            format: glium::texture::ClientFormat::U8U8U8U8,
        }
    }

    fn draw_vec(texture: &glium::Texture2d, data: &Vec<u8>, x: u32, y: u32, data_w: u32, data_h: u32){
        let raw = Self::create_raw(data, data_w, data_h);

        texture.write(
            glium::Rect{left: x, bottom: y, width: data_w, height: data_h}, 
            raw
        );   
    }

    fn inner_redraw(&self, texture: &glium::Texture2d){
        let data = self.data.lock().unwrap();
        let raw = Self::create_raw(&data, self.width, self.height);

        texture.write(
            glium::Rect{
                left: Self::BORDER_SZ, 
                bottom: Self::BORDER_SZ, 
                width: self.width, 
                height: self.height
            }, 
            raw
        );   
    }

    fn init_border_draw(&mut self, texture: &glium::Texture2d, width: u32, height: u32){
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
    
        Self::draw_vec(texture, &line_x, 0, 0, big_width, 1);
        Self::draw_vec(texture, &line_x, 0, 2, big_width, 1);
        Self::draw_vec(texture, &line_x, 0, big_height - 3, big_width, 1);
        Self::draw_vec(texture, &line_x, 0, big_height - 1, big_width, 1);

        Self::draw_vec(texture, &line_y, 0, 0, 1, big_height);
        Self::draw_vec(texture, &line_y, 2, 0, 1, big_height);
        Self::draw_vec(texture, &line_y, big_width - 3, 0, 1, big_height);
        Self::draw_vec(texture, &line_y, big_width - 1, 0, 1, big_height);

        Self::draw_vec(texture, &data, Self::BORDER_SZ, Self::BORDER_SZ, width, height);

        self.data = Arc::new(Mutex::new(data));
    }

    pub fn new(width: u32, height: u32) { Self::new_all(width, height, Self::STD_START_POS) }

    pub fn new_all(width: u32, height: u32, pos: (u32, u32)) {
        let width = width + Self::BORDER_2SZ;
        let height = height + Self::BORDER_2SZ;

        let event_loop = EventLoop::<Event<()>>::with_user_event();
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
        
        let mut thread_win = Self{
            need_redraw: Arc::new(AtomicBool::new(true)),
            data: Arc::new(Mutex::new(vec![])),
            width,
            height,
            drawer: None,
        };

        
        thread_win.init_border_draw(&texture, width, height);
        let inner_win = thread_win.clone();

        let id = display.gl_window().window().id();
        let proxy = event_loop.create_proxy();
        thread_win.drawer = Some(DrawThreadWin{ can_thread_draw: false, id, proxy });

        std::thread::spawn(move||{
            loop {
                std::thread::sleep(std::time::Duration::from_millis(2500));
                thread_win.start_draw_frame();
                thread_win.clear_color(Color{r: 0x00, g: 0xFF, b: 0x00, a: 0xFF});
                thread_win.end_draw_frame();
                
                std::thread::sleep(std::time::Duration::from_millis(2500));
                thread_win.start_draw_frame();
                thread_win.clear_color(Color{r: 0xFF, g: 0x00, b: 0x00, a: 0xFF});
                thread_win.end_draw_frame();
            }
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if inner_win.need_redraw.load(Ordering::Relaxed) {
                inner_win.inner_redraw(&texture);
                let frame = display.draw();
                texture
                    .as_surface()
                    .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
                frame.finish().unwrap();
                inner_win.need_redraw.store(false, Ordering::Relaxed);
            }

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    inner_win.inner_redraw(&texture);
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