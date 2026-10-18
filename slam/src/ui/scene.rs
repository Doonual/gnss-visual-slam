use easy_imgui_window::{easy_imgui as imgui,};
use glam::Mat4;
use easy_imgui::Vector2;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use glam::Vec3;
use std::rc::Rc;
use easy_imgui_sys;

use glow::HasContext;
use crate::ui;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex { pos: [f32; 3], col: [f32; 4] }

const VS: &str = r#"#version 330 core
layout(location=0) in vec3 pos;
layout(location=1) in vec4 col;
uniform mat4 mvp;
out vec4 v_col;
void main() { gl_Position = mvp * vec4(pos, 1.0); gl_PointSize = 4.0; v_col = col; }"#;

const FS: &str = r#"#version 330 core
in vec4 v_col; out vec4 o;
void main() { o = v_col; }"#;

pub struct Scene3D {
	prog: glow::Program,
	mvp_loc: Option<glow::UniformLocation>,
	vao: glow::VertexArray,
	vbo: glow::Buffer,
	fbo: Option<glow::Framebuffer>,
	pub tex: Option<glow::Texture>,
	depth: Option<glow::Renderbuffer>,
	size: (i32, i32),
	points: Vec<Vertex>,
	lines: Vec<Vertex>,
	tris: Vec<Vertex>,
}

impl Scene3D {
	pub fn new(gl: Rc<glow::Context>) -> Self {
		unsafe {
			let prog = gl.create_program().unwrap();
			for (ty, src) in [(glow::VERTEX_SHADER, VS), (glow::FRAGMENT_SHADER, FS)] {
				let sh = gl.create_shader(ty).unwrap();
				gl.shader_source(sh, src);
				gl.compile_shader(sh);
				assert!(gl.get_shader_compile_status(sh), "{}", gl.get_shader_info_log(sh));
				gl.attach_shader(prog, sh);
			}
			gl.link_program(prog);
			assert!(gl.get_program_link_status(prog), "{}", gl.get_program_info_log(prog));
			let mvp_loc = gl.get_uniform_location(prog, "mvp");
			
			let vao = gl.create_vertex_array().unwrap();
			let vbo = gl.create_buffer().unwrap();
			gl.bind_vertex_array(Some(vao));
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
			let stride = std::mem::size_of::<Vertex>() as i32;
			gl.enable_vertex_attrib_array(0);
			gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
			gl.enable_vertex_attrib_array(1);
			gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, 12);
			gl.bind_vertex_array(None);
			
			Self { prog, mvp_loc, vao, vbo, fbo: None, tex: None, depth: None,
			size: (0, 0), points: vec![], lines: vec![], tris: vec![] }
		}
	}
	
	// --- immediate-mode API: call these every frame ---
	#[allow(dead_code)]
	pub fn point(&mut self, point: glam::Vec3, r: u8, g: u8, b: u8, a: u8) {
		self.points.push(Vertex { pos: [point.x, point.y, point.z], col: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0] });
	}
	#[allow(dead_code)]
	pub fn line(&mut self, point_a: glam::Vec3, point_b: glam::Vec3, r: u8, g: u8, b: u8, a: u8) {
		self.lines.push(Vertex { pos: [point_a.x, point_a.y, point_a.z], col: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0] });
		self.lines.push(Vertex { pos: [point_b.x, point_b.y, point_b.z], col: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0] });
	}
	#[allow(dead_code)]
	pub fn tri(&mut self, point_a: glam::Vec3, point_b: glam::Vec3, point_c: glam::Vec3, r: u8, g: u8, b: u8, a: u8) {
		for p in [[point_a.x, point_a.y, point_a.z], [point_b.x, point_b.y, point_b.z], [point_c.x, point_c.y, point_c.z]] { self.tris.push(Vertex { pos: p, col: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0] }); }
	}
	
	fn resize(&mut self, gl: Rc<glow::Context>, w: i32, h: i32) {
		unsafe {
			
			if let Some(f) = self.fbo.take() { gl.delete_framebuffer(f); }
			if let Some(t) = self.tex.take() { gl.delete_texture(t); }
			if let Some(d) = self.depth.take() { gl.delete_renderbuffer(d); }
			
			let tex = gl.create_texture().unwrap();
			gl.bind_texture(glow::TEXTURE_2D, Some(tex));
			gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA8 as i32, w, h, 0,
			glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(None));
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			
			let depth = gl.create_renderbuffer().unwrap();
			gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
			gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT24, w, h);
			
			let fbo = gl.create_framebuffer().unwrap();
			gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
			gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0,
			glow::TEXTURE_2D, Some(tex), 0);
			gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT,
			glow::RENDERBUFFER, Some(depth));
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			
			self.fbo = Some(fbo); self.tex = Some(tex); self.depth = Some(depth);
			self.size = (w, h);
		}
	}
	
	pub fn render(&mut self, gl_in: Rc<glow::Context>, w: i32, h: i32, mvp: Mat4) {
		
		let gl = gl_in.clone();
		
		if (w, h) != self.size { self.resize(gl.clone(), w, h); }
		unsafe {
			
			gl.bind_framebuffer(glow::FRAMEBUFFER, self.fbo);
			gl.viewport(0, 0, w, h);
			gl.clear_color(0.08, 0.08, 0.1, 1.0); // alpha 0.0 if you want it see-through
			gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl.enable(glow::DEPTH_TEST);
			gl.enable(glow::PROGRAM_POINT_SIZE);
			gl.use_program(Some(self.prog));
			gl.uniform_matrix_4_f32_slice(self.mvp_loc.as_ref(), false, &mvp.to_cols_array());
			gl.bind_vertex_array(Some(self.vao));
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
			for (verts, mode) in [(&self.tris, glow::TRIANGLES),
			(&self.lines, glow::LINES),
			(&self.points, glow::POINTS)] {
				if verts.is_empty() { continue; }
				gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,
				bytemuck::cast_slice(verts), glow::STREAM_DRAW);
				gl.draw_arrays(mode, 0, verts.len() as i32);
			}
			// put things back the way imgui's renderer expects
			gl.disable(glow::DEPTH_TEST);
			gl.bind_vertex_array(None);
			gl.use_program(None);
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
		}
		self.points.clear(); self.lines.clear(); self.tris.clear();
	}
	
	
	
	pub fn imgui_draw_scene_only(&mut self, gl: Rc<glow::Context>, imgui: &imgui::Ui<crate::App>, selected: &mut bool, vertical_fov: f32, pivot_centre: glam::Vec3, camera_rotation: &mut glam::Quat, camera_zoom: &mut f32) {
		
		let aspect_ratio = 1.0;
		let near_clip_dist = 0.001;
		let far_clip_dist = 100.0;
		let view_matrix = glam::camera::lh::proj::opengl::perspective(vertical_fov, aspect_ratio, near_clip_dist, far_clip_dist);
		
		let camera_position = camera_rotation.mul_vec3(glam::Vec3{x: 0.0, y: 0.0, z: -*camera_zoom}) + pivot_centre;
		let transformation_matrix = Mat4::from_rotation_translation(*camera_rotation, camera_position).inverse();
		
		let avail = imgui.get_content_region_avail();
		
		let mut width = avail.x - 1.0;
		if width > 400.0 {
			//width = 400.0;
		}
		
		
		self.render(gl.clone(), width as i32, width as i32, view_matrix * transformation_matrix);
		
		
		if let Some(tex_id) = self.tex {
			
			let tex_id = Renderer::map_tex(tex_id);
			let tex_ref = TextureRef::Id(tex_id);
			
			let mut border_width = 2.0;
			if *selected == true {
				border_width = 4.0;
			}
			
			
			imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
				imgui.with_push((imgui::StyleVar::FrameBorderSize, imgui::StyleValue::F32(border_width)), || {
					imgui.image_button_config(imgui::id("Lol"), tex_ref, Vector2{x: width, y: width}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
				});
			});
			
			
			if *selected == true {
				unsafe {
					// Dodgy, probably better to find the actual id but fuck it
					easy_imgui_sys::ImGui_SetKeyOwner(easy_imgui_sys::ImGuiKey::ImGuiKey_MouseWheelY, 19832398 as u32 ,0);
				}
			}
			
			if imgui.io().MouseClicked[0] == true {
				if imgui.is_item_active() == true {
					*selected = true;
				}
				else {
					*selected = false;
				}
			}
			
			if *selected == true {
				// Zoom
				*camera_zoom = *camera_zoom * f32::exp(imgui.io().MouseWheel * -0.1);
			}
			if *selected == true {
				
				
				if imgui.io().MouseDown[0] == true {
					
					// Rotation
					*camera_rotation = *camera_rotation * glam::Quat::from_euler(glam::EulerRot::XYZ, imgui.io().MouseDelta.y * 0.01, imgui.io().MouseDelta.x * 0.01, 0.0);
					let right_vec = camera_rotation.mul_vec3(Vec3{x: 1.0, y: 0.0, z: 0.0});
					let flat_right_vec = Vec3{x: right_vec.x, y: 0.0, z: right_vec.z}.normalize();
					*camera_rotation = glam::Quat::from_rotation_arc(right_vec, flat_right_vec) * *camera_rotation;
					
					
				}
				
			}
			
		}
		else {
			eprintln!("Render texture missing");
		}
		
	}
	
}

pub trait DrawSceneGizmo {
	fn draw_scene_gizmo(&self, scene: &mut Scene3D);
}



pub struct SceneImguiState {
	pub has_focus: bool,
	pub vertical_fov: f32,
	pub pivot_centre: glam::Vec3,
	pub camera_rotation: glam::Quat,
	pub camera_zoom: f32,
}

impl Default for SceneImguiState {
	fn default() -> Self {
		
		
		let mut camera_rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, glam::Vec3{x: 0.5, y: -0.2, z: 0.5}.normalize());
		let right_vec = camera_rotation.mul_vec3(Vec3{x: 1.0, y: 0.0, z: 0.0});
		let flat_right_vec = Vec3{x: right_vec.x, y: 0.0, z: right_vec.z}.normalize();
		camera_rotation = glam::Quat::from_rotation_arc(right_vec, flat_right_vec) * camera_rotation;
		
		SceneImguiState {
			has_focus: false,
			vertical_fov: 45.0,
			pivot_centre: glam::Vec3::ZERO,
			camera_rotation: camera_rotation,
			camera_zoom: 20.0,
		}
	}
}



impl ui::ImguiInspect for Scene3D {
	
	type ImguiState = SceneImguiState;
	
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>, gl: Rc<glow::Context>) {
		
		//imgui.table_config("Test table".into(), 2).with(|| {
			
			//imgui.table_next_row(easy_imgui::TableRowFlags::None, 0.0);
			//imgui.table_next_column();
			
			//let avail = imgui.get_content_region_avail();
			
			
			//imgui.set_next_item_width(avail.x);
			//imgui.text("Hi from column 1");
			
			//imgui.table_next_column();
			
			
			
			//imgui.list_box("###Trackers".into(), self.trackers.len() as i32, 0..self.trackers.len(), |i| format!("Tracker #{i}"), &mut ui_state.tracker_index_selected);
			//imgui.combo("###Trackers".into(), 0..self.trackers.len(), |i| format!("Tracker #{i}"), &mut ui_state.tracker_index_selected);
			
			
			self.imgui_draw_scene_only(gl, imgui, &mut ui_state.has_focus, ui_state.vertical_fov, ui_state.pivot_centre, &mut ui_state.camera_rotation, &mut ui_state.camera_zoom);
			
			
			//imgui.table_next_column();
			
		//});
		
		
	}
	
}



