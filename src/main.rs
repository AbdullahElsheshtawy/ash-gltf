mod app;
use anyhow::Result;
use app::App;
use winit::{
    dpi::PhysicalSize,
    event_loop::{self, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "trace"));
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_title("Hello, Vulkan");
    let app = App::new(window.build(&event_loop)?, event_loop)?;

    app.run()?;

    Ok(())
}
