mod app;
use anyhow::Result;
use app::App;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "trace"));
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_title("Hello, Vulkan");
    let mut app = App::new(window.build(&event_loop)?)?;

    app.run(event_loop);

    Ok(())
}
