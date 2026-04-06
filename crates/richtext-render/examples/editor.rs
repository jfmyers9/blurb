use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use parley::{FontContext, LayoutContext};
use richtext_core::history::History;
use richtext_core::markdown;
use richtext_core::node::Node;
use richtext_core::schema::NodeType;
use richtext_core::state::{EditorState, Selection};
use richtext_render::cursor::CursorState;
use richtext_render::input;
use richtext_render::layout::{self, Color, LayoutBlock};
use richtext_render::mouse::MouseState;
use richtext_render::render::Renderer;
use softbuffer::Surface;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ButtonSource, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const BG: Color = Color::rgb(255, 255, 255);

struct EditorApp {
    editor_state: EditorState,
    history: History,
    cursor: CursorState,
    mouse: MouseState,
    font_cx: FontContext,
    layout_cx: LayoutContext<Color>,
    renderer: Renderer,
    blocks: Vec<LayoutBlock>,
    win: Option<WinState>,
    modifiers: ModifiersState,
    needs_layout: bool,
    scale_factor: f32,
}

struct WinState {
    window: Arc<dyn Window>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
}

impl EditorApp {
    fn relayout(&mut self, width: f32) {
        self.blocks = layout::layout_document_scaled(
            &self.editor_state.doc,
            &mut self.font_cx,
            &mut self.layout_cx,
            width,
            self.scale_factor,
        );
        self.needs_layout = false;
    }
}

impl ApplicationHandler for EditorApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.win.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("richtext editor")
            .with_surface_size(LogicalSize::new(800u32, 600u32));
        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(attrs).expect("create window"));
        let context = softbuffer::Context::new(window.clone()).expect("context");
        let surface = Surface::new(&context, window.clone()).expect("surface");

        self.scale_factor = window.scale_factor() as f32;
        let size = window.surface_size();
        let logical_width = size.width.max(100) as f32 / self.scale_factor;
        self.relayout(logical_width - 20.0);

        self.win = Some(WinState { window, surface });
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _wid: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::SurfaceResized(size) => {
                let logical_width = size.width.max(100) as f32 / self.scale_factor;
                self.relayout(logical_width - 20.0);
                if let Some(ws) = self.win.as_ref() {
                    ws.window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let size = self.win.as_ref().unwrap().window.surface_size();
                let w = size.width.max(1);
                let h = size.height.max(1);

                self.win
                    .as_mut()
                    .unwrap()
                    .surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .expect("resize");

                if self.needs_layout {
                    let logical_width = w as f32 / self.scale_factor;
                    self.relayout(logical_width - 20.0);
                }

                let mut pixmap = Pixmap::new(w, h).expect("pixmap");
                pixmap.fill(tiny_skia::Color::from_rgba8(BG.r, BG.g, BG.b, 255));

                // Selection highlight (behind text)
                let sel = &self.editor_state.selection;
                if !sel.is_collapsed() {
                    richtext_render::selection::draw_selection(
                        &mut pixmap.as_mut(),
                        &self.blocks,
                        sel.from(),
                        sel.to(),
                    );
                }

                // Text
                self.renderer
                    .paint_blocks(&mut pixmap.as_mut(), &self.blocks);

                // Caret
                if self.editor_state.selection.is_collapsed() && self.cursor.is_visible() {
                    richtext_render::cursor::draw_caret(
                        &mut pixmap.as_mut(),
                        &self.blocks,
                        self.editor_state.selection.head,
                    );
                }

                let ws = self.win.as_mut().unwrap();
                let mut buffer = ws.surface.buffer_mut().expect("buffer");
                let px = pixmap.data();
                for i in 0..(w * h) as usize {
                    let off = i * 4;
                    buffer[i] =
                        ((px[off] as u32) << 16) | ((px[off + 1] as u32) << 8) | px[off + 2] as u32;
                }
                buffer.present().expect("present");
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                let super_key = self.modifiers.meta_key();
                let shift = self.modifiers.shift_key();
                let mut changed = false;

                match &event.logical_key {
                    Key::Character(ch) if super_key => match ch.as_str() {
                        "z" if shift => {
                            changed = input::redo(&mut self.editor_state, &mut self.history);
                        }
                        "z" => {
                            changed = input::undo(&mut self.editor_state, &mut self.history);
                        }
                        _ => {}
                    },
                    Key::Character(ch) if !super_key && !self.modifiers.control_key() => {
                        changed = input::insert_text(
                            &mut self.editor_state,
                            &mut self.history,
                            &self.blocks,
                            ch.as_str(),
                        );
                    }
                    Key::Named(NamedKey::Backspace) => {
                        changed = input::delete_backward(
                            &mut self.editor_state,
                            &mut self.history,
                            &self.blocks,
                        );
                    }
                    Key::Named(NamedKey::Delete) => {
                        changed = input::delete_forward(
                            &mut self.editor_state,
                            &mut self.history,
                            &self.blocks,
                        );
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        input::move_left(&mut self.editor_state, &self.blocks, shift);
                        changed = true;
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        input::move_right(&mut self.editor_state, &self.blocks, shift);
                        changed = true;
                    }
                    _ => {}
                }

                if changed {
                    self.needs_layout = true;
                    self.cursor.reset();
                }

                if let Some(ws) = self.win.as_ref() {
                    ws.window.request_redraw();
                }
            }

            WindowEvent::PointerButton {
                state: ElementState::Pressed,
                button: ButtonSource::Mouse(MouseButton::Left),
                position,
                ..
            } => {
                let x = position.x as f32 * self.scale_factor;
                let y = position.y as f32 * self.scale_factor;
                let pos = layout::hit_test(&self.blocks, x, y);
                self.mouse.pressed = true;
                self.mouse.drag_anchor = Some(pos);
                if self.modifiers.shift_key() {
                    self.editor_state.selection.head = pos;
                } else {
                    self.editor_state.selection = Selection::cursor(pos);
                }
                self.cursor.reset();
                if let Some(ws) = self.win.as_ref() {
                    ws.window.request_redraw();
                }
            }

            WindowEvent::PointerButton {
                state: ElementState::Released,
                button: ButtonSource::Mouse(MouseButton::Left),
                ..
            } => {
                self.mouse.release();
            }

            WindowEvent::PointerMoved { position, .. } => {
                let x = position.x as f32 * self.scale_factor;
                let y = position.y as f32 * self.scale_factor;

                if let Some((anchor, head)) = self.mouse.drag(&self.blocks, x, y) {
                    self.editor_state.selection = Selection::new(anchor, head);
                    self.cursor.reset();
                    if let Some(ws) = self.win.as_ref() {
                        ws.window.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.cursor.tick() {
            if let Some(ws) = self.win.as_ref() {
                ws.window.request_redraw();
            }
        }
        let ms = self.cursor.ms_until_toggle().max(16);
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + std::time::Duration::from_millis(ms),
        ));
    }

    fn resumed(&mut self, _: &dyn ActiveEventLoop) {}
}

fn main() {
    let doc = if let Some(path) = std::env::args().nth(1) {
        let md = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read {path}: {e}");
            std::process::exit(1);
        });
        markdown::parse_markdown(&md)
    } else {
        Node::new(
            NodeType::Doc,
            vec![Node::new(
                NodeType::Paragraph,
                vec![Node::text("Type here...", Vec::new())],
            )],
        )
    };

    let editor_state = EditorState::new(doc, Selection::cursor(1));

    EventLoop::new()
        .expect("event loop")
        .run_app(EditorApp {
            editor_state,
            history: History::new(),
            cursor: CursorState::new(),
            mouse: MouseState::new(),
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            renderer: Renderer::new(),
            blocks: Vec::new(),
            win: None,
            modifiers: ModifiersState::empty(),
            needs_layout: true,
            scale_factor: 1.0,
        })
        .expect("run");
}
