use std::num::NonZeroU32;
use std::sync::Arc;

use parley::FontContext;
use parley::LayoutContext;
use richtext_core::markdown;
use richtext_render::layout::{self, Color};
use richtext_render::render::Renderer;
use softbuffer::Surface;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const BG_COLOR: Color = Color::rgb(255, 255, 255);

struct App {
    doc: richtext_core::node::Node,
    state: Option<AppState>,
    font_cx: FontContext,
    layout_cx: LayoutContext<Color>,
    renderer: Renderer,
}

struct AppState {
    window: Arc<dyn Window>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("richtext-render demo")
            .with_surface_size(LogicalSize::new(800u32, 600u32));
        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(attrs).expect("create window"));
        let context = softbuffer::Context::new(window.clone()).expect("create context");
        let surface = Surface::new(&context, window.clone()).expect("create surface");
        self.state = Some(AppState { window, surface });
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let state = self.state.as_mut().unwrap();
                let size = state.window.surface_size();
                let w = size.width.max(1);
                let h = size.height.max(1);

                state
                    .surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .expect("resize surface");

                let scale = state.window.scale_factor() as f32;
                let logical_width = w as f32 / scale;
                let blocks = layout::layout_document_scaled(
                    &self.doc,
                    &mut self.font_cx,
                    &mut self.layout_cx,
                    logical_width - 20.0,
                    scale,
                );

                let mut pixmap = Pixmap::new(w, h).expect("create pixmap");
                pixmap.fill(tiny_skia::Color::from_rgba8(
                    BG_COLOR.r, BG_COLOR.g, BG_COLOR.b, 255,
                ));

                self.renderer.paint_blocks(&mut pixmap.as_mut(), &blocks);

                let mut buffer = state.surface.buffer_mut().expect("get buffer");
                let px_data = pixmap.data();
                for i in 0..(w * h) as usize {
                    let offset = i * 4;
                    let r = px_data[offset] as u32;
                    let g = px_data[offset + 1] as u32;
                    let b = px_data[offset + 2] as u32;
                    buffer[i] = (r << 16) | (g << 8) | b;
                }
                buffer.present().expect("present buffer");
            }
            _ => {
                if let Some(state) = self.state.as_ref() {
                    state.window.request_redraw();
                }
            }
        }
    }

    fn resumed(&mut self, _event_loop: &dyn ActiveEventLoop) {}
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run -p richtext-render --example demo -- <path/to/file.md>");
        std::process::exit(1);
    });

    let md = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", path, e);
        std::process::exit(1);
    });

    let doc = markdown::parse_markdown(&md);

    let event_loop = EventLoop::new().expect("create event loop");
    event_loop
        .run_app(App {
            doc,
            state: None,
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            renderer: Renderer::new(),
        })
        .expect("run event loop");
}
