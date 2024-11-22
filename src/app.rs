use log::info;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};
#[derive(Debug)]
pub struct App {
    window: Option<winit::window::Window>,
    window_attribs: WindowAttributes,
}

impl App {
    pub fn new(window_attribs: WindowAttributes) -> Self {
        Self {
            window: None,
            window_attribs,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(self.window_attribs.clone())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        if window_id == window.id() {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }

                WindowEvent::RedrawRequested => window.request_redraw(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match key {
                    KeyCode::Escape => event_loop.exit(),
                    KeyCode::F11 => {
                        if window.fullscreen().is_some() {
                            window.set_fullscreen(None);
                        } else {
                            window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                    }
                    _ => {}
                },

                _ => {}
            }
        }
    }
}
