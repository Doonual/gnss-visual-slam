use glam;
use crate::ui;
use easy_imgui as imgui;
use std::rc::Rc;

pub struct PixelTracker {
	
	pub uv_position: glam::Vec2,
	pub point_index: i32,
	
}

impl PixelTracker {
	
	pub fn draw_tracker_over_image(&self, imgui: &imgui::Ui<crate::App>, top_left: imgui::Vector2, bottom_right: imgui::Vector2) {
		
		let centre = imgui::vec2(top_left.x + (bottom_right.x - top_left.x) * self.uv_position.x, top_left.y + (bottom_right.y - top_left.y) * self.uv_position.y);
		
		//imgui.window_draw_list().add_line(top_left, bottom_right, imgui::Color{r: 1.0, g: 0.0, b: 0.0, a: 1.0}, 1.0);
		let cross_size = 5.0;
		let border = 2.5;
		let thickness = 2.5;
		
		
		imgui.window_draw_list().add_line(centre + imgui::vec2(-border - cross_size, -border - cross_size), centre + imgui::vec2(border + cross_size, border + cross_size), imgui::Color::BLACK, thickness + border * 2.00);
		imgui.window_draw_list().add_line(centre + imgui::vec2(border + cross_size, -border - cross_size), centre + imgui::vec2(-border - cross_size, border + cross_size), imgui::Color::BLACK, thickness + border * 2.0);
		imgui.window_draw_list().add_line(centre + imgui::vec2(-cross_size, -cross_size), centre + imgui::vec2(cross_size, cross_size), imgui::Color::RED, thickness);
		imgui.window_draw_list().add_line(centre + imgui::vec2(cross_size, -cross_size), centre + imgui::vec2(-cross_size, cross_size), imgui::Color::RED, thickness);
		
	}
	
}

impl ui::ImguiInspect for PixelTracker {
	
	type UiState = ();
	
	fn draw_imgui_inspector(&mut self, ui_state_: &mut Self::UiState, imgui: &imgui::Ui<crate::App>, gl: Rc<glow::Context>) {
		
		let mut pos_arr = self.uv_position.to_array();
		imgui.drag_float_2_config("Position (UV)".into(), &mut pos_arr).speed(0.001).build();
		self.uv_position = glam::Vec2::from_array(pos_arr);
		
		imgui.input_int_config("Point index".into(), &mut self.point_index).build();
		
		
	}
	
}
