mod state;
mod texture;

use std::sync::Arc;
use winit::{application::ApplicationHandler, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, platform::windows::WindowAttributesExtWindows, window::{Window, WindowAttributes, WindowButtons, WindowId, WindowLevel}};
use crate::state::State;
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        println!("Window oluşturuluyor...");
        let attr = WindowAttributes::default()
            .with_resizable(false)
            .with_skip_taskbar(true)
            .with_transparent(true)
            .with_decorations(false)
            .with_active(false)
            .with_content_protected(true)
            .with_maximized(true)
            .with_enabled_buttons(WindowButtons::empty());

        let window = Arc::new(event_loop.create_window(attr).unwrap());
        // Tıklama geçişini (Click-through)
        let _ = window.set_cursor_hittest(false);
        window.set_ime_allowed(false);
        window.set_window_level(WindowLevel::AlwaysOnTop);

        // State başlatılıyor
        match State::new(Arc::clone(&window)) {
            Ok(state) => {
                self.state = Some(state);
                self.window = Some(window);
                println!("Window ve Vulkan State başarıyla oluşturuldu.");
            }
            Err(e) => {
                eprintln!("Kritik Hata - State oluşturulamadı: {}", e);
                event_loop.exit();
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Kapatma isteği alındı, çıkılıyor...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // State ve Window yüklüyse render al
                if let Some(state) = self.state.as_mut() {
                    match state.render() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Render Hatası: {:?}", e),
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(state) = self.state.as_mut() {
                        state.resize(size);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            println!("Escape tuşuna basıldı, çıkılıyor...");
                            event_loop.exit();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Sürekli akış ve akıcı bir overlay için pencerenin yeniden çizilmesini tetikliyoruz
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    println!("Event loop oluşturuluyor.");
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    println!("Event loop çalıştırılıyor.");
    event_loop.run_app(&mut App::default())?;

    Ok(())
}