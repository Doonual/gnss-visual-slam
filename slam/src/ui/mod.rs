use easy_imgui_window::{easy_imgui as imgui,};
use image::ImageFormat;
use json::JsonValue;
use crate::{file::{DecodeJsonError, Saveable}, slam::{self, SlamState, SlamStateGizmoState, keyframe::SlamKeyframeSource}, ui::{images_window::ImagesWindow, scene_window::SceneWindow}};

pub mod scene_window;
pub mod images_window;
pub mod solver_panel;


pub trait ImguiInspect {
	type ImguiState: Default;
	fn draw_imgui_inspector(&mut self, ui_state: &mut Self::ImguiState, imgui: &imgui::Ui<crate::App>);
}


#[derive(Default)]
pub struct UiRoot {
	
	pub slam_state: SlamState,
	slam_state_gizmo_state: SlamStateGizmoState,
	//slam_state_ui_state: SlamStateUiState,
	
	images_window: ImagesWindow,
	scene_window: SceneWindow,
	
}


impl UiRoot {
	
	pub fn render(&mut self, imgui: &imgui::Ui<crate::App>) {
		
		
		imgui.window_config("Images".into()).with(|| {
			self.images_window.imgui_draw_images_window(&mut self.slam_state, imgui);
		});
		imgui.window_config("Scene".into()).with(|| {
			self.scene_window.imgui_draw_scene_window(&mut self.slam_state, imgui);
		});
		
		
	}
	
}

impl Saveable for UiRoot {
	
	fn to_json(&self) -> json::JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		json_obj["slam state"] = self.slam_state.to_json();
		json_obj["slam gizmo state"] = self.slam_state_gizmo_state.to_json();
		json_obj["images window"] = self.images_window.to_json();
		json_obj["scene window"] = self.scene_window.to_json();
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		
		let slam_state = SlamState::from_json(&json_value["slam state"])?;
		let slam_state_gizmo_state = SlamStateGizmoState::from_json(&json_value["slam gizmo state"])?;
		let images_window = ImagesWindow::from_json(&json_value["images window"])?;
		let scene_window = SceneWindow::from_json(&json_value["scene window"])?;
		
		Ok(Self {
			slam_state,
			slam_state_gizmo_state,
			images_window,
			scene_window,
		})
		
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let slam_state = SlamState::from_json_or_default(&json_value["slam state"]);
		let slam_state_gizmo_state = SlamStateGizmoState::from_json_or_default(&json_value["slam gizmo state"]);
		let images_window = ImagesWindow::from_json_or_default(&json_value["images window"]);
		let scene_window = SceneWindow::from_json_or_default(&json_value["scene window"]);
		
		Self {
			slam_state,
			slam_state_gizmo_state,
			images_window,
			scene_window,
		}
	}
}


pub fn image_extensions() -> Vec<&'static str> {
	ImageFormat::all()
	.filter(|f| f.reading_enabled())   // only formats compiled in via cargo features
	.flat_map(|f| f.extensions_str().iter().copied())
	.collect()
}

