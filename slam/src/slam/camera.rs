use easy_imgui as imgui;
use std::rc::Rc;
use glam::{Vec2, Vec3, Quat, Mat4};
use crate::ui;
use crate::slam::pixel_tracker::PixelTracker;


pub struct SlamCamera {
	view_matrix: glam::Mat4,
	pub position: glam::Vec3,
	pub rotation: glam::Quat,
}

impl Default for SlamCamera {
	fn default() -> Self {
		
		let iphone14_view_matrix = [
			0.7515, 0.0000, 0.5000, 0.0000,
			0.0000, 1.0020, 0.5000, 0.0000,
			0.0000, 0.0000, 1.0000, 0.0000,
			0.0000, 0.0000, 0.0000, 1.0000,
		] as [f32; 16];
		
		SlamCamera {
			view_matrix: Mat4::from_rows_array(&iphone14_view_matrix),
			position: Vec3::ZERO,
			rotation: Quat::IDENTITY,
		}
	}
}

impl SlamCamera {
	pub fn get_transformation_matrix(&self) -> Mat4 {
		Mat4::from_rotation_translation(self.rotation, self.position)
	}
	pub fn get_aspect_ratio(&self) -> f32 {
		self.view_matrix.row(1)[1] / self.view_matrix.row(0)[0]
	}
	pub fn get_view_dist(&self) -> f32 {
		1.0 / self.view_matrix.row(1)[1]
	}
	pub fn project_uv(&self, uv: Vec2) -> Vec3 {
		let aspect_ratio = self.get_aspect_ratio();
		let dist = self.get_view_dist();
		let local_pos = Vec3::Z * dist + Vec3::X * (uv.x - 0.0) + Vec3::Y * (1.0 - uv.y - 0.0);
		self.rotation.mul_vec3(self.view_matrix.inverse().project_point3(local_pos))
	}
	pub fn project_tracker(&self, tracker: &PixelTracker) -> Vec3 {
		 self.project_uv(tracker.uv_position)
	}
}


impl ui::ImguiInspect for SlamCamera {
	type ImguiState = ();
	fn draw_imgui_inspector(&mut self, imgui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>, gl: Rc<glow::Context>) {
		
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
		if imgui.drag_float_3_config("Position".into(), &mut pos_arr).speed(0.01).build() == true {
			self.position = glam::Vec3::from_array(pos_arr);
		}
		if imgui.drag_float_4_config("Rotation".into(), &mut rot_arr).speed(0.01).build() == true {
			self.rotation = glam::Quat::from_array(rot_arr).normalize();
		}
		
		
		
	}
}

impl ui::scene::DrawSceneGizmo for SlamCamera {
	fn draw_scene_gizmo(&self, scene: &mut ui::scene::Scene3D) {
		
		let right = self.rotation.mul_vec3(Vec3::X);
		let up = self.rotation.mul_vec3(Vec3::Y);
		let forward = self.rotation.mul_vec3(Vec3::Z);
		
		let aspect_ratio = self.get_aspect_ratio();
		let vertical_fov = 2.0 * f32::atan(0.5 * self.view_matrix.row(1)[1]);
		let dist = 1.0 / self.view_matrix.row(1)[1];
		
		let rect_top_left = self.position + forward * dist + up * 0.5 - right * 0.5 * aspect_ratio;
		let rect_top_right = self.position + forward * dist + up * 0.5 + right * 0.5 * aspect_ratio;
		let rect_bottom_left = self.position + forward * dist - up * 0.5 - right * 0.5 * aspect_ratio;
		let rect_bottom_right = self.position + forward * dist - up * 0.5 + right * 0.5 * aspect_ratio;
		
		scene.line(rect_top_left, rect_top_right, 255, 255, 255, 255);
		scene.line(rect_top_right, rect_bottom_right, 255, 255, 255, 255);
		scene.line(rect_bottom_right, rect_bottom_left, 255, 255, 255, 255);
		scene.line(rect_bottom_left, rect_top_left, 255, 255, 255, 255);
		
		scene.line(self.position, rect_top_left, 255, 255, 255, 255);
		scene.line(self.position, rect_top_right, 255, 255, 255, 255);
		scene.line(self.position, rect_bottom_left, 255, 255, 255, 255);
		scene.line(self.position, rect_bottom_right, 255, 255, 255, 255);
		
		
	}
}
