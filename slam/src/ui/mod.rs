use easy_imgui_window::{easy_imgui as imgui,};
use easy_imgui::Vector2;
use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;
use std::rc::Rc;
use glam;

pub mod scene;

use crate::slam;
use scene::DrawSceneGizmo;


pub struct Ui {
	
	scene: scene::Scene3D,
	scene_state: scene::SceneImguiState,
	
	slam_keyframe_ui_state: slam::keyframe::SlamKeyframeUiState,
	
	show_demo: bool,
	label_image_index: i32,
	
}



impl Ui {
	pub fn new(gl: Rc<glow::Context>) -> Self {
		
		Ui {
			
			
			scene: scene::Scene3D::new(gl),
			scene_state: scene::SceneImguiState::default(),
			
			slam_keyframe_ui_state: slam::keyframe::SlamKeyframeUiState::default(),
			
			show_demo: false,
			label_image_index: 0,
		}
	}
	
	pub fn render(&mut self, gl: Rc<glow::Context>, imgui: &imgui::Ui<crate::App>, slam_state: &mut slam::SlamState) {
		
		
		imgui.window_config("Images".into()).with(|| {
			
			let mut float_workaround = self.label_image_index as f32;
			let image_index_changed = imgui.slider_float_config("Image index".into(), &mut float_workaround).range(0.0, slam_state.slam_keyframes.len() as f32 - 1.0).display_format(easy_imgui::FloatFormat::F(0)).build();
			self.label_image_index = float_workaround as i32;
			if image_index_changed == true {
				self.slam_keyframe_ui_state = slam::keyframe::SlamKeyframeUiState::default();
			}
			
			
			if let Some(slam_keyframe) = slam_state.slam_keyframes.get_mut(self.label_image_index as usize) {
				
				let id = self as *const Self as usize;
				imgui.child_config(format!("keyframe_inspector_{0}", id).into()).size(imgui::vec2(-1.0, 0.0)).child_flags(imgui::ChildFlags::Borders | imgui::ChildFlags::AutoResizeY).with(|| {
					slam_keyframe.draw_imgui_inspector(&mut self.slam_keyframe_ui_state, imgui, gl.clone());
				});
			};
			
			
			imgui.checkbox("Show ImGui Demo".into(), &mut self.show_demo);
			if self.show_demo == true {
				imgui.show_demo_window(Some(&mut self.show_demo));
			}
			
			
		});
		
		
		imgui.window_config("Scene".into()).with(|| {
			
			let dim = 5;
			let scale = 10.0;
			for x in -dim..dim+1 {
				let val = (x as f32) / dim as f32;
				
				self.scene.line(glam::vec3(val, 0.0, -1.0) * scale, glam::vec3(val, 0.0, 1.0) * scale, 200, 200, 200, 80);
				self.scene.line(glam::vec3(-1.0, 0.0, val) * scale, glam::vec3(1.0, 0.0, val) * scale, 200, 200, 200, 80);
			}
			
			for slam_keyrame in &mut slam_state.slam_keyframes {
				slam_keyrame.draw_scene_gizmo(&mut self.scene);
			}
			
			self.scene.draw_imgui_inspector(&mut self.scene_state, imgui, gl.clone());
			
		});
		
		
	}
	
}


pub trait ImguiInspect {
	
	type ImguiState: Default;
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>, gl: Rc<glow::Context>);
}

