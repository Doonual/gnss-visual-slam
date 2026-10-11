use easy_imgui_window::{easy_imgui as imgui,};
use std::rc::Rc;
use glam::Mat4;
use easy_imgui::Vector2;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use glam::Vec3;

use crate::render_scene;


pub struct Ui {
	scene: render_scene::Scene3D,
	camera_rotation: glam::Quat,
	camera_pivot: glam::Vec3,
	camera_zoom: f32,
	show_demo: bool,
}


impl Ui {
	pub fn new(gl: Rc<glow::Context>) -> Self {
		Ui {
			scene: render_scene::Scene3D::new(gl),
			camera_rotation: glam::Quat::IDENTITY,
			camera_pivot: glam::Vec3{x: 0.0, y: 0.0, z: 0.0},
			camera_zoom: 3.0,
			show_demo: false,
		}
	}
	
	pub fn render(&mut self, imgui: &imgui::Ui<crate::App>) {
		
		
		
		imgui.window_config("Test Window".into()).with(|| {
			imgui.checkbox("Show ImGui Demo".into(), &mut self.show_demo);
			
			if self.show_demo == true {
				imgui.show_demo_window(None);
			}
			
			let pivot_pos = &mut self.camera_pivot.to_array();
			imgui.drag_float_3_config("Camera pivot".into(), pivot_pos).speed(0.01).build();
			self.camera_pivot = Vec3::from_array(pivot_pos.clone());
			
			imgui.slider_float_config("Camera zoom".into(), &mut self.camera_zoom).range(0.1, 10.0).flags(imgui::SliderFlags::Logarithmic).build();
			
			let avail = imgui.get_content_region_avail();
			
			let dim = 15;
			for x in -dim..dim+1 {
				
				let x_pos = (x as f32) / dim as f32;
				
				self.scene.line([x_pos, 0.0, -1.0], [x_pos, 0.0, 1.0], [0.8, 0.8, 0.8, 1.0]);
				self.scene.line([-1.0, 0.0, x_pos], [1.0, 0.0, x_pos], [0.8, 0.8, 0.8, 1.0]);
				
			}
			
			let vertical_fov = 45.0;
			let aspect_ratio = 1.0;
			let near_clip_dist = 0.01;
			let far_clip_dist = 10.0;
			let view_matrix = glam::camera::rh::proj::opengl::perspective(vertical_fov, aspect_ratio, near_clip_dist, far_clip_dist);
			
			let camera_position = self.camera_rotation.mul_vec3(glam::Vec3{x: 0.0, y: 0.0, z: self.camera_zoom}) + self.camera_pivot;
			let transformation_matrix = Mat4::from_rotation_translation(self.camera_rotation, camera_position).inverse();
			
			
			self.scene.render(avail.x as i32, avail.x as i32, view_matrix * transformation_matrix);
			
			
			if let Some(tex_id) = self.scene.tex {
				let tex_id = Renderer::map_tex(tex_id);
				let tex_ref = TextureRef::Id(tex_id);
				
				imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
					imgui.image_button_config(imgui::id("Lol"), tex_ref, Vector2{x: avail.x, y: avail.x}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
				});
				if imgui.is_item_hovered() == true {
					// Zoom
					self.camera_zoom = self.camera_zoom * f32::exp(imgui.io().MouseWheel * -0.1);
				}
				if imgui.is_item_active() == true {
					
					
					if imgui.io().MouseDown[0] == true {
						
						// Rotation
						self.camera_rotation = self.camera_rotation * glam::Quat::from_euler(glam::EulerRot::XYZ, imgui.io().MouseDelta.y * -0.01, imgui.io().MouseDelta.x * -0.01, 0.0);
						let right_vec = self.camera_rotation.mul_vec3(Vec3{x: 1.0, y: 0.0, z: 0.0});
						let flat_right_vec = Vec3{x: right_vec.x, y: 0.0, z: right_vec.z}.normalize();
						self.camera_rotation = glam::Quat::from_rotation_arc(right_vec, flat_right_vec) * self.camera_rotation;
						
						
					}
					
				}
				
			}
			else {
				eprintln!("Render texture missing");
			}
			
			
			imgui.text("Hi");
		});
		
	}
	
}


