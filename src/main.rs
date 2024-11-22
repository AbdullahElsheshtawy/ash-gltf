mod app;
use anyhow::Result;
use app::App;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() -> Result<()> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "trace");
    env_logger::init_from_env(env);
    let event_loop = EventLoop::new()?;
    let window_attribs = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_title("Hello, Vulkan");

    let mut app = App::new(window_attribs);

    event_loop.run_app(&mut app)?;

    Ok(())
}
