use glam;
use crate::{file::DecodeJsonError, ui};
use easy_imgui as imgui;

use crate::file;
use json::{self, JsonValue};

pub struct PixelTracker {
	
	pub uv_position: glam::Vec2,
	pub point_index: i32,
	
}

impl PixelTracker {
	
	pub fn draw_tracker_over_image(&self, imgui: &imgui::Ui<crate::App>, top_left: imgui::Vector2, bottom_right: imgui::Vector2) {
		
		let centre = imgui::vec2(top_left.x + (bottom_right.x - top_left.x) * self.uv_position.x, top_left.y + (bottom_right.y - top_left.y) * self.uv_position.y);
		
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
	
	type ImguiState = ();
	
	fn draw_imgui_inspector(&mut self, _ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>) {
		
		let mut pos_arr = self.uv_position.to_array();
		imgui.drag_float_2_config("Position (UV)".into(), &mut pos_arr).speed(0.001).build();
		self.uv_position = glam::Vec2::from_array(pos_arr);
		
		imgui.input_int_config("Point index".into(), &mut self.point_index).build();
		
		
	}
	
}

impl file::Saveable for PixelTracker {
	
	fn to_json(&self) -> json::JsonValue {
		
		let mut tracker_json = json::JsonValue::new_object();
		tracker_json["uv position"] = self.uv_position.to_json();
		tracker_json["point index"] = self.point_index.into();
		tracker_json
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(point) = json_value["point index"].as_i32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "point index".into() });};
		
		Ok(PixelTracker {
			uv_position: glam::Vec2::from_json(&json_value["uv position"])?,
			point_index: point,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let point = json_value["point index"].as_i32().unwrap_or(0);
		
		PixelTracker {
			uv_position: glam::Vec2::from_json_or_default(&json_value["uv position"]),
			point_index: point,
		}
	}
}

