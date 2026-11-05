use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use easy_imgui as imgui;
use glam::Vec2;
use json::JsonValue;
use crate::file::{DecodeJsonError, Saveable};
use crate::slam::{SlamState, SlamStateGizmoState};
use crate::slam::keyframe::{SlamKeyframeSource};
use crate::scene::{DrawSceneGizmo, Scene3D, SceneInspectState};
use crate::ui::solver_panel::SolverPanel;
use crate::ui::{ImguiInspect};


#[derive(Default)]
pub struct SceneWindow {
	
	scene: Scene3D,
	scene_ui_state: SceneInspectState,
	slam_state_gizmo_state: SlamStateGizmoState,
	
	slam_solver_panel: SolverPanel,
	
	object_focussed: Option<u64>,
	
}


impl SceneWindow {
	
	pub fn imgui_draw_scene_window(&mut self, slam_state: &mut SlamState, imgui: &imgui::Ui<crate::App>) {
		
		slam_state.create_missing_enabled_keyframe_entries();
		
		
		imgui.table_config("Scene layout table".into(), 2).with(|| {
			
			imgui.table_setup_column("Settings column", imgui::TableColumnFlags::WidthStretch, 0.33, 0);
			imgui.table_setup_column("Scene column", imgui::TableColumnFlags::WidthStretch, 0.66, 1);
			imgui.table_next_row(easy_imgui::TableRowFlags::None, 0.0);
			
			
			imgui.table_next_column();
			
			imgui.separator_text("SLAM Solver");
			
			self.slam_solver_panel.draw_imgui_panel(slam_state, imgui);
			
			
			imgui.separator_text(format!("SLAM Keyframes ({})", slam_state.slam_keyframes.len()));
			imgui.child_config("Camera list child".into()).child_flags(imgui::ChildFlags::Borders).size(imgui::vec2(-1.0, 300.0)).with(|| {
				
				for i in 0..slam_state.slam_keyframes.len() {
					
					let slam_keyframe = &mut slam_state.slam_keyframes[i];
					
					let mut hash = DefaultHasher::default();
					slam_keyframe.hash(&mut hash);
					let slam_keyframe_hash = hash.finish();
					
					
					let cursor_pos = imgui.get_cursor_screen_pos();
					
					let mut selected = false;
					if let Some(object_focussed_actual) = self.object_focussed {
						selected = object_focussed_actual == slam_keyframe_hash;
					}
					
					let selectable_height = imgui.get_frame_height_with_spacing() - imgui.style().ItemSpacing.y;
					if imgui.selectable_config(format!("##keyframe_{i}_selectable").into()).selected(selected).size(imgui::vec2(0.0, selectable_height)).flags(imgui::SelectableFlags::AllowOverlap).build() {
						
						if let Some(currently_focussed_hash) = self.object_focussed && currently_focussed_hash == slam_keyframe_hash {
							self.object_focussed = None;
						}
						else {
							self.object_focussed = Some(slam_keyframe_hash);
						}
					}
					imgui.set_cursor_screen_pos(cursor_pos);
					
					if let Some(mut keyframe_enabled) = slam_state.enabled_keyframes.get_mut(&slam_keyframe.get_hash_value()) {
						imgui.checkbox(format!("##{i}").into(), keyframe_enabled);
						
					}
					
					
					imgui.same_line();
					
					let mut keyframe_gizmo_col = imgui::Color {
						r: slam_keyframe.gizmo_col[0] as f32 / 255.0,
						g: slam_keyframe.gizmo_col[1] as f32 / 255.0,
						b: slam_keyframe.gizmo_col[2] as f32 / 255.0,
						a: slam_keyframe.gizmo_col[3] as f32 / 255.0,
					};
					imgui.color_edit_4_config(format!("Col##{i}").into(), &mut keyframe_gizmo_col).flags(imgui::ColorEditFlags::NoLabel | imgui::ColorEditFlags::NoInputs).build();
					slam_keyframe.gizmo_col[0] = (keyframe_gizmo_col.r * 255.0) as u8;
					slam_keyframe.gizmo_col[1] = (keyframe_gizmo_col.g * 255.0) as u8;
					slam_keyframe.gizmo_col[2] = (keyframe_gizmo_col.b * 255.0) as u8;
					slam_keyframe.gizmo_col[3] = (keyframe_gizmo_col.a * 255.0) as u8;
					
					imgui.same_line();
					
					imgui.text(format!("Keyframe {:#X}...", slam_keyframe_hash / 0x1000000000000).as_str());
					
					
					
					//if let Some(keyframe_image_source) = &slam_keyframe.image_source && let Some(current_keyframe_ui_state) = slam_state_ui_state.slam_keyframe_ui_states.get_mut(keyframe_image_source) {
						//imgui.checkbox(format!("Shown##{i}").into(), &mut current_keyframe_ui_state.shown);
					//}
					//else {
						//imgui.text("SLAM Keyframe not Serialisable");
					//}
					
					
				}
				
				
			});
			
			for keyframe in &mut slam_state.slam_keyframes {
				
				let mut hash = DefaultHasher::default();
				keyframe.hash(&mut hash);
				let keyframe_hash = hash.finish();
				
				
				if self.object_focussed.is_none() {continue;}
				if let Some(object_focussed) = self.object_focussed && object_focussed != keyframe_hash {continue;}
				
				imgui.separator_text(format!("Selected Keyframe {:#X}...", keyframe_hash / 0x1000000000000));
				keyframe.camera_pose.draw_imgui_inspector(&mut (), imgui);
				
				self.scene_ui_state.pivot_centre = keyframe.camera_pose.position + keyframe.project_uv(Vec2{x: 0.5, y: 0.5}) * keyframe.get_view_dist() * 0.5;
				
			}
			
			
			imgui.table_next_column();
			
			slam_state.draw_scene_gizmo(&mut self.slam_state_gizmo_state, &mut self.scene);
			self.scene.draw_imgui_inspector(&mut self.scene_ui_state, imgui);
			
			imgui.table_next_column();
			
		});
		
	}
	
}

impl Saveable for SceneWindow {
	fn to_json(&self) -> json::JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		json_obj["scene ui state"] = self.scene_ui_state.to_json();
		json_obj["slam state gizmo state"] = self.slam_state_gizmo_state.to_json();
		json_obj["slam solver panel"] = self.slam_solver_panel.to_json();
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let scene_ui_state = SceneInspectState::from_json(&json_value["scene ui state"])?;
		let slam_state_gizmo_state = SlamStateGizmoState::from_json(&json_value["slam state gizmo state"])?;
		let slam_solver_panel = SolverPanel::from_json(&json_value["slam solver panel"])?;
		
		Ok(Self {
			
			scene: Scene3D::default(),
			object_focussed: None,
			
			scene_ui_state,
			slam_state_gizmo_state,
			slam_solver_panel,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let scene_ui_state = SceneInspectState::from_json_or_default(&json_value["scene ui state"]);
		let slam_state_gizmo_state = SlamStateGizmoState::from_json_or_default(&json_value["slam state gizmo state"]);
		let slam_solver_panel = SolverPanel::from_json_or_default(&json_value["slam solver panel"]);
		
		Self {
			
			scene: Scene3D::default(),
			object_focussed: None,
			
			scene_ui_state,
			slam_state_gizmo_state,
			slam_solver_panel,
		}
	}
}


struct SlamKeyframeUiState {
	show_gizmo: bool,
	col: [u8; 4],
}

impl Default for SlamKeyframeUiState {
	fn default() -> Self {
		SlamKeyframeUiState {
			show_gizmo: true,
			col: [255, 255, 255, 255],
		}
	}
}
