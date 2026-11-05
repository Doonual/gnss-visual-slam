use std::fmt;
use std::hash::DefaultHasher;
use std::hash::Hasher;
use easy_imgui as imgui;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use glam::Mat4;
use glam::Quat;
use imgui::Vector2;

use glam::{Vec2, Vec3};

use glow;
use glow::HasContext;
use json::JsonValue;
use std::hash::Hash;


use crate::file::DecodeJsonError;
use crate::file::Saveable;
use crate::scene::DrawSceneGizmo;
use crate::scene::Scene3D;
use crate::slam::pixel_tracker::PixelTracker;

use crate::file;
use crate::ui::ImguiInspect;
use json;


pub struct SlamKeyframe {
	
	pub camera_pose: CameraPose,
	
	pub image_source: Option<SlamKeyframeSource>,
	pub image: Option<glow::Texture>,
	pub image_width: Option<u32>,
	pub image_height: Option<u32>,
	
	pub trackers: Vec<PixelTracker>,
	
	pub gizmo_shown: bool,
	pub gizmo_col: [u8; 4],
	//pub tracker_gizmo_col: [u8; 4],
	
}

impl SlamKeyframe {
	
	pub fn upload_image_data(&mut self) -> Result<(), CreateSlamKeyframeError> {
		
		let Some(slam_keyframe_source) = &self.image_source else {return Err(CreateSlamKeyframeError::NoSource);};
		
		match slam_keyframe_source {
			SlamKeyframeSource::ImageFile(path) => {
				
				let image_reader = match image::open(path) {
					Err(e) => return Err(CreateSlamKeyframeError::ImageOpen(e)),
					Ok(r) => r,
				};
				
				
				let image = image_reader.flipv().to_rgba8();
				let (w, h) = image.dimensions();
				
				unsafe {
					
					let gl = crate::gl();
					
					let tex = match gl.create_texture() {
						Err(e) => return Err(CreateSlamKeyframeError::CreateTexture(e)),
						Ok(r) => r,
					};
					
					
					gl.bind_texture(glow::TEXTURE_2D, Some(tex));
					
					gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
					gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
					gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
					gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
					
					gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
					gl.tex_image_2d(
						glow::TEXTURE_2D,
						0,
						glow::RGBA8 as i32,
						w as i32,
						h as i32,
						0,
						glow::RGBA,
						glow::UNSIGNED_BYTE,
						glow::PixelUnpackData::Slice(Some(image.as_raw())),
					);
					
					self.image = Some(tex);
					self.image_width = Some(w);
					self.image_height = Some(h);
					
					Ok(())
					
				}
			},
		}
		
		
		
		
	}
	
	fn imgui_draw_image(&mut self, imgui: &imgui::Ui<crate::App>) {
		
		if self.image.is_none() && let Err(e) = self.upload_image_data() {
			eprintln!("Failed to upload image data: {e}");
		}
		
		let image_width = match self.image_width {
			Some(img) => img,
			None => return,
		};
		let image_height = match self.image_height {
			Some(img) => img,
			None => return,
		};
		
		let width = imgui.get_content_region_avail().x;
		let height = width * image_height as f32 / image_width as f32;
		
		
		match self.image {
			Some(texture) => {
				let tex_id = Renderer::map_tex(texture);
				let tex_ref = TextureRef::Id(tex_id);
				
				let border_width = 2.0;
				let top_left = imgui.get_cursor_screen_pos();
				
				imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
					imgui.with_push((imgui::StyleVar::FrameBorderSize, imgui::StyleValue::F32(border_width)), || {
						imgui.image_button_config("testimage".into(), tex_ref, Vector2{x: width, y: height}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
					});
				});
				let bottom_right = top_left + imgui.get_item_rect_size();
				
				for tracker in &self.trackers {
					tracker.draw_tracker_over_image(imgui, top_left, bottom_right);
				}
				
				
			},
			None => {
				imgui.text("Texure is fucked");
			}
		}
		
	}
	
	pub fn find_first_free_point_index(&self) -> i32 {
		
		let mut empty_point_index = 0;
		let mut empty_point_index_found = false;
		while !empty_point_index_found {
			
			empty_point_index_found = true;
			for tracker in &self.trackers {
				if tracker.point_index == empty_point_index {
					empty_point_index_found = false;
				}
			}
			if !empty_point_index_found {
				empty_point_index += 1;
			}
		}
		
		empty_point_index
		
	}
	
	
	pub fn get_aspect_ratio(&self) -> f32 {
		self.camera_pose.view_matrix.row(1)[1] / self.camera_pose.view_matrix.row(0)[0]
	}
	
	pub fn get_view_dist(&self) -> f32 {
		1.0 / self.camera_pose.view_matrix.row(1)[1]
	}
	
	pub fn project_uv(&self, uv: Vec2) -> Vec3 {
		let dist = self.get_view_dist();
		let local_pos = Vec3::Z * dist + Vec3::X * (uv.x - 0.0) + Vec3::Y * (1.0 - uv.y - 0.0);
		self.camera_pose.rotation.mul_vec3(self.camera_pose.view_matrix.inverse().project_point3(local_pos))
	}
	
	pub fn project_tracker(&self, tracker: &PixelTracker) -> Vec3 {
		self.project_uv(tracker.uv_position)
	}
	
	
	pub fn get_hash_value(&self) -> u64 {
		let mut hash = DefaultHasher::default();
		self.hash(&mut hash);
		hash.finish()
	}
}

impl ImguiInspect for SlamKeyframe {
	type ImguiState = SlamKeyframeInspectState;
	
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &easy_imgui::Ui<crate::App>) {
		
		
		imgui.table_config("Test table".into(), 2).with(|| {
			
			imgui.table_setup_column("Trackers column", imgui::TableColumnFlags::WidthStretch, 0.33, 0);
			imgui.table_setup_column("Image column", imgui::TableColumnFlags::WidthStretch, 0.66, 1);
			imgui.table_next_row(easy_imgui::TableRowFlags::None, 0.0);
			
			
			imgui.table_next_column();
			// ###########################################
			// ### Table 1st column (Trackers coulumn) ###
			// ###########################################
			
			let avail = imgui.get_content_region_avail();
			
			
			imgui.separator_text("Camera info");
			
			self.camera_pose.draw_imgui_inspector(&mut (), imgui);
			
			
			imgui.separator_text("Trackers");
			
			
			if imgui.button_config("Create new tracker".into()).size(imgui::vec2(-1.0, 0.0)).build() {
				
				let pixel_tracker = PixelTracker{uv_position: glam::Vec2{x: 0.0, y: 0.0}, point_index: self.find_first_free_point_index()};
				self.trackers.push(pixel_tracker);
				ui_state.tracker_index_selected = self.trackers.len() - 1;
			}
			
			imgui.with_disabled(self.trackers.is_empty(), || {
				if imgui.button_config("Delete selected tracker".into()).size(imgui::vec2(-1.0, 0.0)).build() {
					self.trackers.remove(ui_state.tracker_index_selected);
					if ui_state.tracker_index_selected >= self.trackers.len() {
						ui_state.tracker_index_selected = self.trackers.len() - 1;
					}
				}
			});
			
			
			imgui.set_next_item_width(avail.x);
			
			let combo_label_func = |i: usize| {
				if self.trackers.is_empty() {
					return String::from("No trackers");
				}
				format!("Tracker #{0}", i)
			};
			imgui.combo("###Trackers".into(), 0..self.trackers.len(), combo_label_func, &mut ui_state.tracker_index_selected);
			
			
			if let Some(pixel_tracker) = self.trackers.get_mut(ui_state.tracker_index_selected) {
				imgui.separator_text(format!("Tracker #{0} data", ui_state.tracker_index_selected));
				pixel_tracker.draw_imgui_inspector(&mut (), imgui);
			}
			
			
			imgui.table_next_column();
			// ########################################
			// ### Table 2nd column (Image coulumn) ###
			// ########################################
			
			//imgui.separator_text("Image");
			
			
			let top_left = imgui.get_cursor_screen_pos();
			let id = self as *const Self as usize;
			imgui.with_push((imgui::StyleVar::WindowPadding, imgui::StyleValue::Vec2(imgui::vec2(0.0, 0.0))), || {
				imgui.child_config(format!("keyframe_{id}_image").into()).child_flags(imgui::ChildFlags::Borders | imgui::ChildFlags::AutoResizeY).size(imgui::vec2(-1.0, 00.0)).with(|| {
					
					self.imgui_draw_image(imgui);
				});
			});
			
			let window_size =  imgui.get_item_rect_size();
			let mouse_uv = Vec2{x: (imgui.get_mouse_pos().x - top_left.x) / window_size.x, y: (imgui.get_mouse_pos().y - top_left.y) / window_size.y};
			
			if ui_state.create_tracker_on_click && imgui.is_mouse_clicked(imgui::MouseButton::Left) && mouse_uv.x >= 0.0 && mouse_uv.x <= 1.0 && mouse_uv.y >= 0.0 && mouse_uv.y <= 1.0 {
				
				let trakcer = PixelTracker {
					point_index: ui_state.create_tracker_with_index,
					uv_position: mouse_uv,
				};
				
				self.trackers.push(trakcer);
				ui_state.tracker_index_selected = self.trackers.len() - 1;
				
			}
			
			imgui.table_next_column();
			
		});
		
		
		
	}
}

impl DrawSceneGizmo for SlamKeyframe {
	type GizmoState = SlamKeyframeGizmoState;
	fn draw_scene_gizmo(&mut self, gizmo_state: &mut Self::GizmoState, scene: &mut Scene3D) {
		
		if !self.gizmo_shown {return;}
		
		// Draw camera gizmo
		let right = self.camera_pose.rotation.mul_vec3(Vec3::X);
		let up = self.camera_pose.rotation.mul_vec3(Vec3::Y);
		let forward = self.camera_pose.rotation.mul_vec3(Vec3::Z);
		
		let aspect_ratio = self.get_aspect_ratio();
		let dist = 1.0 / self.camera_pose.view_matrix.row(1)[1];
		
		let rect_top_left = self.camera_pose.position + forward * dist + up * 0.5 - right * 0.5 * aspect_ratio;
		let rect_top_right = self.camera_pose.position + forward * dist + up * 0.5 + right * 0.5 * aspect_ratio;
		let rect_bottom_left = self.camera_pose.position + forward * dist - up * 0.5 - right * 0.5 * aspect_ratio;
		let rect_bottom_right = self.camera_pose.position + forward * dist - up * 0.5 + right * 0.5 * aspect_ratio;
		
		scene.line(rect_top_left, rect_top_right, self.gizmo_col);
		scene.line(rect_top_right, rect_bottom_right, self.gizmo_col);
		scene.line(rect_bottom_right, rect_bottom_left, self.gizmo_col);
		scene.line(rect_bottom_left, rect_top_left, self.gizmo_col);
		
		scene.line(self.camera_pose.position, rect_top_left, self.gizmo_col);
		scene.line(self.camera_pose.position, rect_top_right, self.gizmo_col);
		scene.line(self.camera_pose.position, rect_bottom_left, self.gizmo_col);
		scene.line(self.camera_pose.position, rect_bottom_right, self.gizmo_col);
		
		
		
		for tracker in &self.trackers {
			
			let projected_tracker = self.project_tracker(tracker);
			
			let dist = self.get_view_dist() / self.camera_pose.rotation.inverse().mul_vec3(projected_tracker).z;
			
			scene.line(self.camera_pose.position, self.camera_pose.position + projected_tracker * dist, [255, 0, 0, 255]);
			
			if gizmo_state.show_projection_lines {
				scene.line(self.camera_pose.position + projected_tracker * dist, self.camera_pose.position + projected_tracker * 10.0, [127, 0, 0, 255]);
			}
			
			let right = self.camera_pose.rotation.mul_vec3(Vec3::X);
			let up = self.camera_pose.rotation.mul_vec3(Vec3::Y);
			let forward = self.camera_pose.rotation.mul_vec3(Vec3::Z);
			let view_dist = self.get_view_dist();
			
			let scaled_uv = (tracker.uv_position - Vec2{x: 0.5, y: 0.5}) * Vec2{x: self.get_aspect_ratio(), y: 1.0};
			let screen_pos = right * scaled_uv.x - up * scaled_uv.y;
			
			let cross_size = 0.03;
			scene.line(self.camera_pose.position + screen_pos + forward * view_dist + (right + up) * cross_size, self.camera_pose.position + screen_pos + forward * view_dist - (right + up) * cross_size, [255, 0, 0, 255]);
			scene.line(self.camera_pose.position + screen_pos + forward * view_dist + (right - up) * cross_size, self.camera_pose.position + screen_pos + forward * view_dist - (right - up) * cross_size, [255, 0, 0, 255]);
			
			
		}
	}
}


impl file::Saveable for SlamKeyframe {
	
	fn to_json(&self) -> json::JsonValue {
		
		let mut slam_keyframe_json = json::JsonValue::new_object();
		
		if let Some(image_source) = &self.image_source {
			slam_keyframe_json["image source"] = image_source.to_json();
		}
		
		slam_keyframe_json["view matrix"] = self.camera_pose.view_matrix.to_json();
		slam_keyframe_json["position"] = self.camera_pose.position.to_json();
		slam_keyframe_json["rotation"] = self.camera_pose.rotation.to_json();
		
		let mut trackers_json = json::JsonValue::new_array();
		for tracker in &self.trackers {
			if let Err(e) = trackers_json.push(tracker.to_json()) {
				eprintln!("Failed to push to vec: {e}");
			}
		}
		slam_keyframe_json["trackers"] = trackers_json;
		
		slam_keyframe_json["gizmo_shown"] = self.gizmo_shown.into();
		slam_keyframe_json["gizmo_r"] = self.gizmo_col[0].into();
		slam_keyframe_json["gizmo_g"] = self.gizmo_col[1].into();
		slam_keyframe_json["gizmo_b"] = self.gizmo_col[2].into();
		slam_keyframe_json["gizmo_a"] = self.gizmo_col[3].into();
		
		slam_keyframe_json
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let image_source = SlamKeyframeSource::from_json(&json_value["image source"])?;
		
		
		let view_matrix = Mat4::from_json(&json_value["view matrix"])?;
		let position = Vec3::from_json(&json_value["position"])?;
		let rotation = Quat::from_json(&json_value["rotation"])?;
		
		let mut trackers = Vec::new();
		let trackers_json = &json_value["trackers"];
		for i in 0..trackers_json.len() {
			let tracker_json = &trackers_json[i];
			trackers.push(PixelTracker::from_json(&tracker_json)?);
		}
		trackers.reverse();
		
		
		
		let Some(gizmo_shown) = json_value["gizmo_shown"].as_bool() else {
			return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "gizmo_shown".into()} );
		};
		let Some(gizmo_col_r) = json_value["gizmo_r"].as_u8() else {
			return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "gizmo_r".into()} );
		};
		let Some(gizmo_col_g) = json_value["gizmo_g"].as_u8() else {
			return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "gizmo_g".into()} );
		};
		let Some(gizmo_col_b) = json_value["gizmo_b"].as_u8() else {
			return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "gizmo_b".into()} );
		};
		let Some(gizmo_col_a) = json_value["gizmo_a"].as_u8() else {
			return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "gizmo_a".into()} );
		};
		
		let camera_pose = CameraPose {
			position,
			rotation,
			view_matrix,
		};
		
		
		Ok(SlamKeyframe {
			
			camera_pose,
			
			image_source: Some(image_source),
			image: None,
			image_height: None,
			image_width: None,
			
			trackers,
			
			gizmo_shown,
			gizmo_col: [gizmo_col_r, gizmo_col_g, gizmo_col_b, gizmo_col_a],
			
		})
		
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let image_source = SlamKeyframeSource::from_json_or_default(&json_value["image source"]);
		
		
		let view_matrix = Mat4::from_json_or_default(&json_value["view matrix"]);
		let position = Vec3::from_json_or_default(&json_value["position"]);
		let rotation = Quat::from_json_or_default(&json_value["rotation"]);
		
		let mut trackers = Vec::new();
		let trackers_json = &json_value["trackers"];
		for i in 0..trackers_json.len() {
			let tracker_json = &trackers_json[i];
			trackers.push(PixelTracker::from_json_or_default(&tracker_json));
		}
		trackers.reverse();
		
		
		
		let gizmo_shown = json_value["gizmo_shown"].as_bool().unwrap_or(true);
		let gizmo_col_r = json_value["gizmo_r"].as_u8().unwrap_or(255);
		let gizmo_col_g = json_value["gizmo_g"].as_u8().unwrap_or(255);
		let gizmo_col_b = json_value["gizmo_b"].as_u8().unwrap_or(255);
		let gizmo_col_a = json_value["gizmo_a"].as_u8().unwrap_or(255);
		
		let camera_pose = CameraPose {
			position,
			rotation,
			view_matrix,
		};
		
		
		SlamKeyframe {
			
			camera_pose,
			
			image_source: Some(image_source),
			image: None,
			image_height: None,
			image_width: None,
			
			trackers,
			
			gizmo_shown,
			gizmo_col: [gizmo_col_r, gizmo_col_g, gizmo_col_b, gizmo_col_a],
			
		}
	}
}

impl Hash for SlamKeyframe {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let val = "SlamKeyframe";
		val.hash(state);
		self.image_source.hash(state);
	}
}
impl std::cmp::PartialEq for SlamKeyframe {
	fn eq(&self, other: &Self) -> bool {
		
		let mut self_hash = DefaultHasher::default();
		let mut other_hash = DefaultHasher::default();
		
		self.hash(&mut self_hash);
		other.hash(&mut other_hash);
		
		self_hash.finish() == other_hash.finish()
		
	}
}
impl std::cmp::Eq for SlamKeyframe {}

pub struct CameraPose {
	pub view_matrix: glam::Mat4,
	pub position: glam::Vec3,
	pub rotation: glam::Quat,
}


impl ImguiInspect for CameraPose {
	type ImguiState = ();
	
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &easy_imgui::Ui<crate::App>) {
		
		imgui.tree_node_config("View matrix".into()).with(|| {
			
			let mut row_0 = self.view_matrix.row(0).to_array();
			let mut row_1 = self.view_matrix.row(1).to_array();
			let mut row_2 = self.view_matrix.row(2).to_array();
			let mut row_3 = self.view_matrix.row(3).to_array();
			
			imgui.input_float_4_config("Row 0".into(), &mut row_0).build();
			imgui.input_float_4_config("Row 1".into(), &mut row_1).build();
			imgui.input_float_4_config("Row 2".into(), &mut row_2).build();
			imgui.input_float_4_config("Row 3".into(), &mut row_3).build();
			
			self.view_matrix = glam::Mat4::from_rows(glam::Vec4::from_array(row_0), glam::Vec4::from_array(row_1), glam::Vec4::from_array(row_2), glam::Vec4::from_array(row_3));
			
		});
		
		let mut pos_arr = self.position.to_array();
		let mut rot_arr = self.rotation.to_array();
		if imgui.drag_float_3_config("Position".into(), &mut pos_arr).speed(0.01).build() {
			self.position = glam::Vec3::from_array(pos_arr);
		}
		if imgui.drag_float_4_config("Rotation".into(), &mut rot_arr).speed(0.01).build() {
			self.rotation = glam::Quat::from_array(rot_arr).normalize();
		}
		
	}
}


#[derive(Default)]
pub struct SlamKeyframeInspectState {
	tracker_index_selected: usize,
	pub create_tracker_on_click: bool,
	pub create_tracker_with_index: i32,
}

impl Saveable for SlamKeyframeInspectState {
	fn to_json(&self) -> JsonValue {
		let mut json_obj = JsonValue::new_object();
		json_obj["tracker index selected"] = self.tracker_index_selected.into();
		json_obj
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(tracker_index_selected) = json_value["tracker index selected"].as_usize() else { return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "name".into() }); };
		
		Ok(Self {
			tracker_index_selected,
			create_tracker_on_click: false,
			create_tracker_with_index: 0,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let tracker_index_selected = json_value["tracker index selected"].as_usize().unwrap_or(0);
		
		Self {
			tracker_index_selected,
			create_tracker_on_click: false,
			create_tracker_with_index: 0,
		}
	}
}

pub struct SlamKeyframeGizmoState {
	pub show_projection_lines: bool,
}
impl Default for SlamKeyframeGizmoState {
	fn default() -> Self {
		SlamKeyframeGizmoState {
			show_projection_lines: true,
		}
	}
}

#[derive(Clone)]
pub enum SlamKeyframeSource {
	ImageFile(String),
}

impl Hash for SlamKeyframeSource {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			SlamKeyframeSource::ImageFile(path) => {
				path.hash(state);
			},
		}
	}
}
impl PartialEq for SlamKeyframeSource {
	fn eq(&self, other: &Self) -> bool {
		let mut hasher_a = DefaultHasher::new();
		let mut hasher_b = DefaultHasher::new();
		
		self.hash(&mut hasher_a);
		other.hash(&mut hasher_b);
		
		hasher_a.finish() == hasher_b.finish()
		
	}
}
impl Eq for SlamKeyframeSource {}

impl Saveable for SlamKeyframeSource {
	
	fn to_json(&self) -> json::JsonValue {
		
		
		let mut source_json = JsonValue::new_object();
		
		match self {
			SlamKeyframeSource::ImageFile(path) => {
				source_json["type"] = "path".into();
				source_json["path"] = path.as_str().into();
				
			},
		}
		source_json
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		
		let Some(source_type) = json_value["type"].as_str() else { return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "type".into() }); };
		
		match source_type {
			
			"path" => {
				let Some(path) = json_value["path"].as_str() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "path".into() });};
				Ok(SlamKeyframeSource::ImageFile(path.into())) 
			},
			other => { Err(DecodeJsonError::UnexpectedValue {json_searched: json_value.clone(), parameter_name: "type".into(), value: format!("{other}").into()}) }
			
		}
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let source_type = json_value["type"].as_str().unwrap_or("path");
		
		match source_type {
			
			"path" => {
				let path = json_value["path"].as_str().unwrap_or("./path/to/.png");
				SlamKeyframeSource::ImageFile(path.into())
			},
			other => { SlamKeyframeSource::ImageFile("./path/to/.png".into()) }
			
		}
	}
	
}

pub enum CreateSlamKeyframeError {
	NoSource,
	ImageOpen(image::ImageError),
	CreateTexture(String),
}

impl fmt::Display for CreateSlamKeyframeError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			
			CreateSlamKeyframeError::NoSource => {
				write!(fmt, "No image data source")
			},
			CreateSlamKeyframeError::ImageOpen(image_error) => {
				write!(fmt, "Failed to open file: {image_error}")
			},
			CreateSlamKeyframeError::CreateTexture(error_string) => {
				write!(fmt, "Failed to create glow texture: {error_string}")
			}
		}
	}
}






pub fn get_iphone14_view_matrix() -> Mat4 {
	
	let iphone14_view_matrix = [
		0.7515, 0.0000, 0.5000, 0.0000,
		0.0000, 1.0020, 0.5000, 0.0000,
		0.0000, 0.0000, 1.0000, 0.0000,
		0.0000, 0.0000, 0.0000, 1.0000,
	] as [f32; 16];
	
	Mat4::from_rows_array(&iphone14_view_matrix)
	
}


