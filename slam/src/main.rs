use easy_imgui_window::{EventFlags, MainWindow, MainWindowWithRenderer, easy_imgui as imgui, glutin, winit};
use glutin::config::Config;
use glutin::prelude::*;
use imgui::Color;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Fullscreen, Window, WindowId};
use x11rb::connection::Connection;
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{ClipOrdering, Rectangle};
use x11rb::rust_connection::RustConnection;
use easy_imgui::WindowFlags;
use std::rc::Rc;

mod render_scene;
mod ui;

// extra pixels around each region, so resize handles and splitters stay grabbable
const PAD: f32 = 4.0;



fn main() {
	
	unsafe { std::env::set_var("WINIT_X11_SCALE_FACTOR", "1"); }
	let event_loop = EventLoop::builder().with_x11().build().unwrap();
	let (conn, _) = x11rb::connect(None).unwrap();

	let mut handler = Handler {
		window: None,
		app: App { gl: None, ui: None, rects: Vec::new() },
		conn,
		applied: Vec::new(),
	};
	event_loop.run_app(&mut handler).unwrap();
}

fn x11_window_id(w: &Window) -> Option<u32> {
	match w.window_handle().ok()?.as_raw() {
		RawWindowHandle::Xlib(h) => Some(h.window as u32),
		RawWindowHandle::Xcb(h) => Some(h.window.get()),
		_ => None,
	}
}

/// Reads imgui's own window list and returns the screen rectangles that should accept input
/// (x, y, w, h). Must be called while the imgui context is current, i.e. inside do_ui.
unsafe fn collect_input_rects() -> Vec<[f32; 4]> {
	use imgui::easy_imgui_sys::*;
	
	let mut out = Vec::new();
	let ctx = unsafe { ImGui_GetCurrentContext() };
	if ctx.is_null() {
		return out;
	}
	let g = unsafe { &*ctx };
	
	for i in 0..g.Windows.Size as usize {
		let w = unsafe { &**g.Windows.Data.add(i) };

		if !(w.Active || w.WasActive) || w.Hidden {
			continue;
		}
		let flags = w.Flags as i32;
		// children are covered by their parent; NoMouseInputs covers tooltips and overlays
		if w.IsExplicitChild {
			continue;
		}
		if flags & (WindowFlags::NoMouseInputs.bits() as i32) != 0 {
			continue;
		}
		// the full-viewport window that hosts our dockspace must stay click-through
		if !w.Name.is_null() {
			let name = unsafe { std::ffi::CStr::from_ptr(w.Name) }.to_string_lossy();
			if name.starts_with("WindowOverViewport_") {
				continue;
			}
		}
		
		// docked windows: use the whole dock node (tab bar included), not just the content
		let (pos, size) = if !w.DockNode.is_null() {
			let node = unsafe { &*w.DockNode };
			(node.Pos, node.Size)
		} else {
			(w.Pos, w.Size)
		};
		if size.x > 0.0 && size.y > 0.0 {
			out.push([pos.x, pos.y, size.x, size.y]);
		}
	}
	out
}

pub struct App {
	gl: Option<Rc<glow::Context>>,
	ui: Option<ui::Ui>,
	rects: Vec<[f32; 4]>, // regions that accept input, refreshed every frame
}

impl imgui::UiBuilder for App {
	fn do_ui(&mut self, imgui: &imgui::Ui<Self>) {
		
		// For docking to edges of monitor
		//imgui.dock_space_over_viewport(
			//0,
			//imgui.get_main_viewport(),
			//imgui::DockNodeFlags::PassthruCentralNode,
			//None,
		//);
		
		if let Some(ui) = &mut self.ui {
			ui.render(imgui);
		}
		
		
		// last, so every window of this frame has been submitted
		self.rects = unsafe { collect_input_rects() };
	}
}

struct Handler {
	window: Option<MainWindowWithRenderer>,
	app: App,
	conn: RustConnection,
	applied: Vec<[i32; 4]>,
}

impl ApplicationHandler for Handler {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let attrs = Window::default_attributes()
		.with_title("SLAM Viewer")
		.with_transparent(true)
		.with_decorations(false)
		.with_fullscreen(Some(Fullscreen::Borderless(None)));
		
		let score = |c: &Config| {
			(
				c.supports_transparency().unwrap_or(false),
				std::cmp::Reverse((c.num_samples(), c.depth_size(), c.stencil_size())),
			)
		};
		let main_window = MainWindow::with_gl_chooser(event_loop, attrs, |a, b| {
			if score(&b) > score(&a) { b } else { a }
		})
		.unwrap();
		
		let mut builder = imgui::ContextBuilder::new();
		builder.set_ini_file_name(Some("slam_viewer.ini"));
		
		let mut window = MainWindowWithRenderer::with_builder(main_window, &builder);
		
		const UI_SCALE: f32 = 1.25;   // set this to your KDE display scale
		let style = window.imgui().style_mut();
		style.scale_all_sizes(UI_SCALE);   // paddings, spacing, rounding, scrollbar size
		style.FontScaleMain = UI_SCALE;    // text size
		
		window.renderer().set_background_color(Some(Color::TRANSPARENT));
		window.imgui().io_mut().enable_docking(true);
		self.app.gl = Some(window.renderer().gl_context().clone());
		self.app.ui = Some(ui::Ui::new(window.renderer().gl_context().clone()));
		self.window = Some(window);
		
	}
	
	fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
		
		let Some(window) = self.window.as_mut() else { return };
		if window.main_window().window().id() != id {
			return;
		}
		
		let res = window.window_event(&mut self.app, &event, EventFlags::empty());
		if res.window_closed {
			event_loop.exit();
			return;
		}
		
		// update the X11 input shape from the regions collected in do_ui
		let w = window.main_window().window();
		let scale = w.scale_factor() as f32;
		//eprintln!("scale_factor = {}", w.scale_factor());
		
		
		let mut key: Vec<[i32; 4]> = self.app.rects.iter().map(|r| {
			let x0 = ((r[0] - PAD).max(0.0) * scale) as i32;
			let y0 = ((r[1] - PAD).max(0.0) * scale) as i32;
			let x1 = ((r[0] + r[2] + PAD) * scale) as i32;
			let y1 = ((r[1] + r[3] + PAD) * scale) as i32;
			[x0, y0, (x1 - x0).max(0), (y1 - y0).max(0)]
		}).collect();
		key.sort();
		key.dedup();
		
		if key != self.applied {
			if let Some(xid) = x11_window_id(w) {
				let rects: Vec<Rectangle> = key.iter().map(|k| Rectangle {
					x: k[0] as i16,
					y: k[1] as i16,
					width: k[2] as u16,
					height: k[3] as u16,
				}).collect();
				let _ = self.conn.shape_rectangles(
					shape::SO::SET, shape::SK::INPUT, ClipOrdering::UNSORTED, xid, 0, 0, &rects,
				);
				let _ = self.conn.flush();
				self.applied = key;
			}
		}
	}
	
	fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
		if let Some(window) = self.window.as_mut() {
			window.new_events();
		}
	}
	
	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		if let Some(window) = self.window.as_mut() {
			window.about_to_wait();
		}
	}
}
