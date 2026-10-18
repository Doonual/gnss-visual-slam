use std::fmt;
use easy_imgui as imgui;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use imgui::Vector2;

use glam::{Vec2, Vec3, Quat, Mat4};



use glow;
use glow::HasContext;
use std::rc::Rc;



use crate::slam::pixel_tracker::PixelTracker;
use crate::slam::camera::SlamCamera;
use crate::ui;
use crate::ui::scene::DrawSceneGizmo;
use crate::ui::scene::Scene3D;


pub struct SlamKeyframe {
	
	pub slam_camera: SlamCamera,
	pub trackers: Vec<PixelTracker>,
	
	
	pub image: Option<glow::Texture>,
	pub image_width: Option<u32>,
	pub image_height: Option<u32>,
	
	
}

pub enum CreateSlamKeyframeError {
	ImageOpenError(image::ImageError),
	CreateTextureError(String),
}



impl fmt::Display for CreateSlamKeyframeError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CreateSlamKeyframeError::ImageOpenError(image_error) => {
				return write!(fmt, "Image open error: {image_error}");
			},
			CreateSlamKeyframeError::CreateTextureError(error_string) => {
				return write!(fmt, "Error creating glow texture: {error_string}");
			}
		}
	}
}



impl SlamKeyframe {
	
	pub fn create_from_file(gl: Rc<glow::Context>, path: &str) -> Result<Self, CreateSlamKeyframeError> {
		
		
		let image_reader = match image::open(path) {
			Err(e) => return Err(CreateSlamKeyframeError::ImageOpenError(e)),
			Ok(r) => r,
		};
		
		
		let image = image_reader.flipv().to_rgba8();
		let (w, h) = image.dimensions();
		
		unsafe {
			
			let tex = match gl.create_texture() {
				Err(e) => return Err(CreateSlamKeyframeError::CreateTextureError(e)),
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
			
			
			Ok(SlamKeyframe {
				image: Some(tex),
				image_width: Some(w),
				image_height: Some(h),
				
				slam_camera: SlamCamera::default(),
				trackers: Vec::new(),
				
			})
			
			
		}
		
		
		
	}
	
	fn imgui_draw_image(&self, imgui: &imgui::Ui<crate::App>) {
		
		let image_width = match self.image_width {
			Some(img) => img,
			None => return,
		};
		let image_height = match self.image_height {
			Some(img) => img,
			None => return,
		};
		
		let mut width = imgui.get_content_region_avail().x;
		if width > 400.0 {
			//width = 400.0;
		}
		let height = width * image_height as f32 / image_width as f32;
		
		imgui.with_push((imgui::StyleVar::WindowPadding, imgui::StyleValue::Vec2(imgui::vec2(0.0, 0.0))), || {
			let id = self as *const Self as usize;
			imgui.child_config(format!("keyframe_image_child_{0}",id).into()).size(imgui::vec2(0.0, height)).child_flags(imgui::ChildFlags::Borders).with(|| {
				
				match self.image {
					Some(texture) => {
						let tex_id = Renderer::map_tex(texture);
						let tex_ref = TextureRef::Id(tex_id);
						
						
						
						let border_width = 2.0;
						
						
						//let top_left = imgui.get_cursor_pos() + imgui.get_window_pos() + imgui::vec2(imgui.get_scroll_x(), imgui.get_scroll_y());
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
			});
		});
		
	}
	
	
}

#[derive(Default)]
pub struct SlamKeyframeUiState {
	tracker_index_selected: usize,
}

impl ui::ImguiInspect for SlamKeyframe {
	
	type ImguiState = SlamKeyframeUiState;
	
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>, gl: Rc<glow::Context>) {
		
		let id = self as *const Self as usize;
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
			self.slam_camera.draw_imgui_inspector(&mut (), imgui, gl.clone());
			
			
			imgui.separator_text("Trackers");
			
			if imgui.button_config("Create new tracker".into()).size(imgui::vec2(-1.0, 0.0)).build() == true {
				let pixel_tracker = PixelTracker{uv_position: glam::Vec2{x: 0.0, y: 0.0}, point_index: 0};
				self.trackers.push(pixel_tracker);
			}
			
			imgui.with_disabled(self.trackers.len() == 0, || {
				if imgui.button_config("Delete selected tracker".into()).size(imgui::vec2(-1.0, 0.0)).build() == true {
					self.trackers.remove(ui_state.tracker_index_selected);
					ui_state.tracker_index_selected = usize::clamp(0, ui_state.tracker_index_selected, self.trackers.len() - 1);
				}
			});
			
			
			imgui.set_next_item_width(avail.x);
			
			let combo_label_func = |i: usize| {
				if self.trackers.len() == 0 {
					return String::from("No trackers");
				}
				return format!("Tracker #{0}", i);
			};
			imgui.combo("###Trackers".into(), 0..self.trackers.len(), combo_label_func, &mut ui_state.tracker_index_selected);
			
			
			if let Some(pixel_tracker) = self.trackers.get_mut(ui_state.tracker_index_selected) {
				imgui.separator_text(format!("Tracker #{0} data", ui_state.tracker_index_selected));
				pixel_tracker.draw_imgui_inspector(&mut (), imgui, gl);
			}
			
			
			imgui.table_next_column();
			// ########################################
			// ### Table 2nd column (Image coulumn) ###
			// ########################################
			
			//imgui.separator_text("Image");
			
			
			self.imgui_draw_image(imgui);
			
			imgui.table_next_column();
			
		});
		
		
		
	}
	
}

impl DrawSceneGizmo for SlamKeyframe {
	fn draw_scene_gizmo(&self, scene: &mut Scene3D) {
		
		self.slam_camera.draw_scene_gizmo(scene);
		
		for tracker in &self.trackers {
			
			//let top_left = Vec3::Z * dist + Vec3::Y * 0.5 + Vec3::X * self.slam_camera.get_aspect_ratio() * 0.5;
			//let right_vec = Vec3::X * 
			
			
			let projected_tracker = self.slam_camera.project_tracker(&tracker);
			
			let dist = self.slam_camera.get_view_dist() / self.slam_camera.rotation.inverse().mul_vec3(projected_tracker).z;
			
			let cross_size = 0.03;
			
			scene.line(self.slam_camera.position, self.slam_camera.position + projected_tracker * dist, 255, 0, 0, 100);
			scene.line(self.slam_camera.position + projected_tracker * dist, self.slam_camera.position + projected_tracker * 10.0, 255, 0, 0, 255);
			
			let right = self.slam_camera.rotation.mul_vec3(Vec3::X);
			let up = self.slam_camera.rotation.mul_vec3(Vec3::Y);
			let forward = self.slam_camera.rotation.mul_vec3(Vec3::Z);
			let view_dist = self.slam_camera.get_view_dist();
			
			let scaled_uv = (tracker.uv_position - Vec2{x: 0.5, y: 0.5}) * Vec2{x: self.slam_camera.get_aspect_ratio(), y: 1.0};
			let screen_pos = right * scaled_uv.x - up * scaled_uv.y;
			
			scene.line(self.slam_camera.position + screen_pos + forward * view_dist + (right + up) * cross_size, self.slam_camera.position + screen_pos + forward * view_dist - (right + up) * cross_size, 255, 0, 0, 255);
			scene.line(self.slam_camera.position + screen_pos + forward * view_dist + (right - up) * cross_size, self.slam_camera.position + screen_pos + forward * view_dist - (right - up) * cross_size, 255, 0, 0, 255);
			
			
		}
		
	}
}
