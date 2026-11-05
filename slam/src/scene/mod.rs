use easy_imgui_window::{easy_imgui as imgui,};
use glam::Mat4;
use easy_imgui::Vector2;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use glam::Quat;
use glam::Vec3;
use json::JsonValue;
use core::f32;
use std::rc::Rc;
use easy_imgui_sys;

use glow::HasContext;
use crate::file::DecodeJsonError;
use crate::file::Saveable;
use crate::ui;

pub mod slam_gizmos;

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

impl Default for Scene3D {
	fn default() -> Self {
		
		let gl = crate::gl();
		
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
}

impl Scene3D {
	
	// --- immediate-mode API: call these every frame ---
	#[allow(dead_code)]
	pub fn point(&mut self, point: glam::Vec3, col: [u8; 4]) {
		self.points.push(Vertex { pos: [point.x, point.y, point.z], col: [col[0] as f32 / 255.0, col[1] as f32 / 255.0, col[2] as f32 / 255.0, col[3] as f32 / 255.0] });
	}
	#[allow(dead_code)]
	pub fn line(&mut self, point_a: glam::Vec3, point_b: glam::Vec3, col: [u8; 4]) {
		self.lines.push(Vertex { pos: [point_a.x, point_a.y, point_a.z], col: [col[0] as f32 / 255.0, col[1] as f32 / 255.0, col[2] as f32 / 255.0, col[3] as f32 / 255.0] });
		self.lines.push(Vertex { pos: [point_b.x, point_b.y, point_b.z], col: [col[0] as f32 / 255.0, col[1] as f32 / 255.0, col[2] as f32 / 255.0, col[3] as f32 / 255.0] });
	}
	#[allow(dead_code)]
	pub fn tri(&mut self, point_a: glam::Vec3, point_b: glam::Vec3, point_c: glam::Vec3, col: [u8; 4]) {
		for p in [[point_a.x, point_a.y, point_a.z], [point_b.x, point_b.y, point_b.z], [point_c.x, point_c.y, point_c.z]] { self.tris.push(Vertex { pos: p, col: [col[0] as f32 / 255.0, col[1] as f32 / 255.0, col[2] as f32 / 255.0, col[3] as f32 / 255.0] }); }
	}
	
	#[allow(dead_code)]
	pub fn circle(&mut self, centre: glam::Vec3, rotation: glam::Quat, radius: f32, col: [u8; 4]) {
		
		let right_vec = rotation * Vec3::X;
		let up_vec = rotation * Vec3::Y;
		
		let circle_res = 32;
		
		for i in 0..circle_res {
			
			let angle = (i as f32 / circle_res as f32) * f32::consts::PI * 2.0;
			let next_angle = ((i + 1) as f32 / circle_res as f32) * f32::consts::PI * 2.0;
			
			let pos_a = centre + f32::cos(angle) * right_vec * radius + f32::sin(angle) * up_vec * radius;
			let pos_b = centre + f32::cos(next_angle) * right_vec * radius + f32::sin(next_angle) * up_vec * radius;
			
			self.line(pos_a, pos_b, col);
			
		}
		
	}
	
	#[allow(dead_code)]
	pub fn sphere(&mut self, centre: glam::Vec3, rotation: glam::Quat, radius: f32, col: [u8; 4]) {
		
		self.circle(centre, rotation, radius, col);
		self.circle(centre, rotation * glam::Quat::from_euler(glam::EulerRot::XYZ, f32::consts::PI / 2.0, 0.0, 0.0), radius, col);
		self.circle(centre, rotation * glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, f32::consts::PI / 2.0, 0.0), radius, col);
		
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
	
	pub fn render(&mut self, w: i32, h: i32, mvp: Mat4) {
		
		let gl = crate::gl();
		
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
	
	pub fn imgui_draw_scene_only(&mut self, ui_state: &mut SceneInspectState, imgui: &imgui::Ui<crate::App>) {
		
		let aspect_ratio = 1.0;
		let near_clip_dist = 0.1;
		let far_clip_dist = 100.0;
		let view_matrix = glam::camera::lh::proj::opengl::perspective(ui_state.vertical_fov, aspect_ratio, near_clip_dist, far_clip_dist);
		
		let camera_position = ui_state.camera_rotation.mul_vec3(glam::Vec3{x: 0.0, y: 0.0, z: -ui_state.camera_zoom}) + ui_state.pivot_centre;
		let transformation_matrix = Mat4::from_rotation_translation(ui_state.camera_rotation, camera_position).inverse();
		
		
		let avail = imgui.get_content_region_avail();
		
		let width = avail.x - 1.0;
		
		self.render(width as i32, width as i32, view_matrix * transformation_matrix);
		
		
		if let Some(tex_id) = self.tex {
			
			let tex_id = Renderer::map_tex(tex_id);
			let tex_ref = TextureRef::Id(tex_id);
			
			let mut border_width = 2.0;
			if ui_state.has_focus {
				border_width = 4.0;
			}
			
			
			imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
				imgui.with_push((imgui::StyleVar::FrameBorderSize, imgui::StyleValue::F32(border_width)), || {
					imgui.image_button_config(imgui::id("Lol"), tex_ref, Vector2{x: width, y: width}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
				});
			});
			
			
			if ui_state.has_focus {
				unsafe {
					// Dodgy, probably better to find the actual id but fuck it
					easy_imgui_sys::ImGui_SetKeyOwner(easy_imgui_sys::ImGuiKey::ImGuiKey_MouseWheelY, 19832398 ,0);
				}
			}
			
			if imgui.io().MouseClicked[0] {
				ui_state.has_focus = imgui.is_item_active();
			}
			
			if ui_state.has_focus {
				// Zoom
				ui_state.camera_zoom *= f32::exp(imgui.io().MouseWheel * -0.1);
			}
			if ui_state.has_focus {
				
				
				if imgui.io().MouseDown[0] {
					
					// Rotation
					ui_state.camera_rotation *= glam::Quat::from_euler(glam::EulerRot::XYZ, imgui.io().MouseDelta.y * 0.01, imgui.io().MouseDelta.x * 0.01, 0.0);
					let right_vec = ui_state.camera_rotation.mul_vec3(Vec3{x: 1.0, y: 0.0, z: 0.0});
					let flat_right_vec = Vec3{x: right_vec.x, y: 0.0, z: right_vec.z}.normalize();
					ui_state.camera_rotation = glam::Quat::from_rotation_arc(right_vec, flat_right_vec) * ui_state.camera_rotation;
					
					
				}
				
				if imgui.io().MouseDown[2] {
					
					let right_delta = -imgui.io().MouseDelta.x * (ui_state.camera_rotation * Vec3::X) * 0.001 * ui_state.camera_zoom;
					let up_delta = imgui.io().MouseDelta.y * (ui_state.camera_rotation * Vec3::Y) * 0.001 * ui_state.camera_zoom;
					ui_state.pivot_centre = ui_state.pivot_centre + right_delta + up_delta;
				}
				
			}
			
		}
		else {
			eprintln!("Render texture missing");
		}
		
	}
	
}

pub trait DrawSceneGizmo {
	type GizmoState: Default;
	fn draw_scene_gizmo(&mut self, gizmo_state: &mut Self::GizmoState, scene: &mut Scene3D);
}



pub struct SceneInspectState {
	has_focus: bool,
	pub vertical_fov: f32,
	pub pivot_centre: glam::Vec3,
	pub camera_rotation: glam::Quat,
	pub camera_zoom: f32,
}

impl Default for SceneInspectState {
	fn default() -> Self {
		
		let mut camera_rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, glam::Vec3{x: 0.5, y: -0.2, z: 0.5}.normalize());
		let right_vec = camera_rotation.mul_vec3(Vec3{x: 1.0, y: 0.0, z: 0.0});
		let flat_right_vec = Vec3{x: right_vec.x, y: 0.0, z: right_vec.z}.normalize();
		camera_rotation = glam::Quat::from_rotation_arc(right_vec, flat_right_vec) * camera_rotation;
		
		SceneInspectState {
			has_focus: false,
			vertical_fov: 45.0,
			pivot_centre: glam::Vec3::ZERO,
			camera_rotation,
			camera_zoom: 20.0,
		}
	}
}

impl Saveable for SceneInspectState {
	
	fn to_json(&self) -> json::JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		json_obj["vertical fov"] = self.vertical_fov.into();
		json_obj["pivot centre"] = self.pivot_centre.to_json();
		json_obj["camera rotation"] = self.camera_rotation.to_json();
		json_obj["camera zoom"] = self.camera_zoom.into();
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(vertical_fov) = json_value["vertical fov"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "vertical fov".into() }); };
		let pivot_centre = Vec3::from_json(&json_value["pivot centre"])?;
		let camera_rotation = Quat::from_json(&json_value["camera rotation"])?;
		let Some(camera_zoom) = json_value["camera zoom"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "camera zoom".into() }); };
		
		Ok(Self {
			has_focus: false,
			vertical_fov,
			pivot_centre,
			camera_rotation,
			camera_zoom,
		})
		
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let vertical_fov = json_value["vertical fov"].as_f32().unwrap_or(45.0);
		let pivot_centre = Vec3::from_json_or_default(&json_value["pivot centre"]);
		let camera_rotation = Quat::from_json_or_default(&json_value["camera rotation"]);
		let camera_zoom = json_value["camera zoom"].as_f32().unwrap_or(20.0);
		
		Self {
			has_focus: false,
			vertical_fov,
			pivot_centre,
			camera_rotation,
			camera_zoom,
		}
	}
}


impl ui::ImguiInspect for Scene3D {
	
	type ImguiState = SceneInspectState;
	
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>) {
		self.imgui_draw_scene_only(ui_state, imgui);
	}
	
}

