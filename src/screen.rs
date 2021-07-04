use pixels::{Error, Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::{Window, WindowBuilder}};

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

pub fn init(rom_name: String) -> Result<(EventLoop<()>, Pixels<Window>, Window), Error> {

    let event_loop = EventLoop::new();
    
    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);
        WindowBuilder::new()
            .with_title(format!("DMG EMU {}", rom_name))
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    Ok((event_loop, pixels, window))
}