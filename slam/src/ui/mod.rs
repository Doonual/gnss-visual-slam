use easy_imgui_window::{easy_imgui as imgui,};
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
		}
	}
	
	pub fn render(&mut self, gl: Rc<glow::Context>, imgui: &imgui::Ui<crate::App>, slam_state: &mut slam::SlamState) {
		
		
		imgui.window_config("Images".into()).with(|| {
			imgui.checkbox("Show ImGui Demo".into(), &mut self.show_demo);
			if self.show_demo == true {
				imgui.show_demo_window(Some(&mut self.show_demo));
			}
			
			if let Some(image) = slam_state.slam_keyframes.get(self.label_image_index as usize) {
				image.imgui_inspect(imgui);
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


