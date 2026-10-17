use easy_imgui_window::{easy_imgui as imgui,};
use easy_imgui::Vector2;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use std::rc::Rc;
use glam;

mod scene;

use crate::slam;


pub struct Ui {
	scene: scene::Scene3D,
	scene_selected: bool,
	camera_rotation: glam::Quat,
	//camera_pivot: glam::Vec3,
	camera_zoom: f32,
	
	show_demo: bool,
	label_image_index: i32,
	pub tracker_index_selected: usize,
}



impl Ui {
	pub fn new(gl: Rc<glow::Context>) -> Self {
		
		Ui {
			scene: scene::Scene3D::new(gl),
			scene_selected: false,
			camera_rotation: glam::Quat::IDENTITY,
			//camera_pivot: glam::Vec3{x: 0.0, y: 0.0, z: 0.0},
			camera_zoom: 3.0,
			show_demo: false,
			label_image_index: 0,
			tracker_index_selected: 0,
		}
	}
	
	pub fn render(&mut self, gl: Rc<glow::Context>, imgui: &imgui::Ui<crate::App>, slam_state: &mut slam::SlamState) {
		
		
		imgui.window_config("Images".into()).with(|| {
			imgui.checkbox("Show ImGui Demo".into(), &mut self.show_demo);
			if self.show_demo == true {
				imgui.show_demo_window(Some(&mut self.show_demo));
			}
			
			
			if let Some(slam_keyframe) = slam_state.slam_keyframes.get_mut(self.label_image_index as usize) {
				slam_keyframe.imgui_inspect(imgui, self);
			};
			let mut float_workaround = self.label_image_index as f32;
			imgui.slider_float_config("Image index".into(), &mut float_workaround).range(0.0, slam_state.slam_keyframes.len() as f32 - 1.0).display_format(easy_imgui::FloatFormat::F(0)).build();
			self.label_image_index = float_workaround as i32;
			
			let dim = 15;
			for x in -dim..dim+1 {
				let x_pos = (x as f32) / dim as f32;
				self.scene.line([x_pos, 0.0, -1.0], [x_pos, 0.0, 1.0], [0.8, 0.8, 0.8, 1.0]);
				self.scene.line([-1.0, 0.0, x_pos], [1.0, 0.0, x_pos], [0.8, 0.8, 0.8, 1.0]);
			}
			scene::render(gl.clone(), imgui, &mut self.scene_selected, &mut self.scene, 45.0, glam::Vec3::ZERO, &mut self.camera_rotation, &mut self.camera_zoom);
			
			
		});
		
	}
	
}




impl slam::keyframe::SlamKeyframe {
	
	pub fn imgui_inspect(&mut self, imgui: &imgui::Ui<crate::App>, ui_state: &mut Ui) {
		
		imgui.table_config("Test table".into(), 2).with(|| {
			
			imgui.table_next_row(easy_imgui::TableRowFlags::None, 0.0);
			imgui.table_next_column();
			
			let mut avail = imgui.get_content_region_avail();
			
			
			
			imgui.set_next_item_width(avail.x);
			imgui.list_box("###Trackers".into(), self.trackers.len() as i32, 0..self.trackers.len(), |i| format!("Tracker #{i}"), &mut ui_state.tracker_index_selected);
			
			imgui.set_next_item_width(avail.x);
			if imgui.button("Create new tracker".into()) == true {
				let pixel_tracker = slam::pixel_tracker::PixelTracker{uv_position: glam::Vec2{x: 0.0, y: 0.0}, point_index: 0};
				self.trackers.push(pixel_tracker);
			}
			
			let disabled = self.trackers.len() == 0;
			imgui.with_disabled(disabled, || {
				imgui.set_next_item_width(avail.x);
				if imgui.button("Delete selected tracker".into()) == true {
					
					self.trackers.remove(ui_state.tracker_index_selected);
					
				}
			});
			
			
			if let Some(pixel_tracker) = self.trackers.get_mut(ui_state.tracker_index_selected) {
				pixel_tracker.imgui_inspect(imgui, ui_state);
			}
			
			
			imgui.table_next_column();
			
			self.imgui_draw_image_only(imgui);
			
			imgui.table_next_column();
			
		});
		
		
	}
	
	fn imgui_draw_image_only(&self, imgui: &imgui::Ui<crate::App>) {
		
		match self.image {
			Some(texture) => {
				let tex_id = Renderer::map_tex(texture);
				let tex_ref = TextureRef::Id(tex_id);
				
				
				let mut width = imgui.get_content_region_avail().x;
				if width > 400.0 {
					width = 400.0;
				}
				let height = width * self.image_height as f32 / self.image_width as f32;
				
				let border_width = 2.0;
				
				
				let top_left = imgui.get_cursor_pos() + imgui.get_window_pos();
				
				imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
					imgui.with_push((imgui::StyleVar::FrameBorderSize, imgui::StyleValue::F32(border_width)), || {
						imgui.image_button_config("testimage".into(), tex_ref, Vector2{x: width, y: height}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
					});
				});
				let bottom_right = top_left + imgui.get_item_rect_size();
				
				for tracker in &self.trackers {
					
					let centre = imgui::vec2(top_left.x + (bottom_right.x - top_left.x) * tracker.uv_position.x, top_left.y + (bottom_right.y - top_left.y) * tracker.uv_position.y);
					
					//imgui.window_draw_list().add_line(top_left, bottom_right, imgui::Color{r: 1.0, g: 0.0, b: 0.0, a: 1.0}, 1.0);
					let cross_size = 5.0;
					let border = 2.5;
					let thickness = 2.5;
					imgui.window_draw_list().add_line(centre + imgui::vec2(-border - cross_size, -border - cross_size), centre + imgui::vec2(border + cross_size, border + cross_size), imgui::Color::BLACK, thickness + border * 2.00);
					imgui.window_draw_list().add_line(centre + imgui::vec2(border + cross_size, -border - cross_size), centre + imgui::vec2(-border - cross_size, border + cross_size), imgui::Color::BLACK, thickness + border * 2.0);
					imgui.window_draw_list().add_line(centre + imgui::vec2(-cross_size, -cross_size), centre + imgui::vec2(cross_size, cross_size), imgui::Color::RED, thickness);
					imgui.window_draw_list().add_line(centre + imgui::vec2(cross_size, -cross_size), centre + imgui::vec2(-cross_size, cross_size), imgui::Color::RED, thickness);
					
					
					
				}
				
				
			},
			None => {
				imgui.text("Texure is fucked");
			}
		}
		
	}
}

impl slam::pixel_tracker::PixelTracker {
	
	pub fn imgui_inspect(&mut self, imgui: &imgui::Ui<crate::App>, ui_state: &mut Ui) {
		
		let mut pos_arr = self.uv_position.to_array();
		imgui.drag_float_2_config("Tracker position".into(), &mut pos_arr).speed(0.001).build();
		self.uv_position = glam::Vec2::from_array(pos_arr);
		
		imgui.input_int_config("Point index".into(), &mut self.point_index).build();
		
		
	}
	
}

