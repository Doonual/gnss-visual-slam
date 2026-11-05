use std::hash::{DefaultHasher, Hash, Hasher};

use easy_imgui as imgui;
use glam::{Quat, Vec3};
use json::JsonValue;

use crate::{file::{DecodeJsonError, Saveable}, slam::{SlamState, keyframe::{self, SlamKeyframeInspectState, SlamKeyframeSource}}, ui::ImguiInspect};

#[derive(Default)]
pub struct ImagesWindow {
	
	create_tracker_on_click: bool,
	create_tracker_with_index: i32,
	
	label_image_index: i32,
	
	current_slam_keyframe_ui_state: SlamKeyframeInspectState,
	
}


impl ImagesWindow {
	
	pub fn imgui_draw_images_window(&mut self, slam_state: &mut SlamState, imgui: &imgui::Ui<crate::App>) {
		
		let id = self as *const Self as usize;
		
		if imgui.button("Open images".into()) {
			
			let extensions = crate::ui::image_extensions();
			let picked = rfd::FileDialog::new()
			.add_filter("Image Files", &extensions)
			.set_title("Select a file")
			.pick_files();
			
			if let Some(picked_actual) = picked {
				for path in picked_actual {
					if let Some(os_path) = path.to_str() {
						let slam_keyframe = keyframe::SlamKeyframe {
							
							camera_pose: keyframe::CameraPose {
								position: Vec3::ZERO,
								rotation: Quat::IDENTITY,
								view_matrix: keyframe::get_iphone14_view_matrix(),
							},
							
							
							image_source: Some(SlamKeyframeSource::ImageFile(os_path.into())),
							trackers: Vec::new(),
							
							image: None,
							image_width: None,
							image_height: None,
							
							gizmo_shown: true,
							gizmo_col: [255, 255, 255, 255],
							
						};
						slam_state.slam_keyframes.push(slam_keyframe);
					}
				}
			}
			
		}
		
		let mut float_workaround = self.label_image_index as f32;
		let image_index_changed = imgui.slider_float_config("Image index".into(), &mut float_workaround).range(0.0, slam_state.slam_keyframes.len() as f32 - 1.0).display_format(easy_imgui::FloatFormat::F(0)).build();
		self.label_image_index = float_workaround as i32;
		if image_index_changed {
			//self.slam_keyframe_ui_state = keyframe::SlamKeyframeUiState::default();
		}
		
		
		imgui.checkbox_config("Create tracker on click".into(), &mut self.create_tracker_on_click).build();
		imgui.input_int_config("Create tracker at index".into(), &mut self.create_tracker_with_index).build();
		
		
		if let Some(slam_keyframe) = slam_state.slam_keyframes.get_mut(self.label_image_index as usize) {
			
			imgui.child_config(format!("keyframe_inspector_{0}", id).into()).size(imgui::vec2(-1.0, 0.0)).child_flags(imgui::ChildFlags::Borders | imgui::ChildFlags::AutoResizeY).with(|| {
				
				self.current_slam_keyframe_ui_state.create_tracker_on_click = self.create_tracker_on_click;
				self.current_slam_keyframe_ui_state.create_tracker_with_index = self.create_tracker_with_index;
				
				slam_keyframe.draw_imgui_inspector(&mut self.current_slam_keyframe_ui_state, imgui);
				
			});
		};
		
	}
	
}

impl Saveable for ImagesWindow {
	fn to_json(&self) -> json::JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		
		json_obj["create tracker on click"] = self.create_tracker_on_click.into();
		json_obj["create tracker with index"] = self.create_tracker_with_index.into();
		json_obj["label image index"] = self.label_image_index.into();
		json_obj["current slam keyframe ui state"] = self.current_slam_keyframe_ui_state.to_json();
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, crate::file::DecodeJsonError> {
		
		let Some(create_tracker_on_click) = json_value["create tracker on click"].as_bool() else { return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "create tracker on click".into() }); };
		let Some(create_tracker_with_index) = json_value["create tracker with index"].as_i32() else { return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "create tracker with index".into() }); };
		let Some(label_image_index) = json_value["label image index"].as_i32() else { return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "label image index".into() }); };
		let current_slam_keyframe_ui_state = SlamKeyframeInspectState::from_json(&json_value["current slam keyframe ui state"])?;
		
		Ok(Self {
			create_tracker_with_index,
			create_tracker_on_click,
			label_image_index,
			current_slam_keyframe_ui_state
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let create_tracker_on_click = json_value["create tracker on click"].as_bool().unwrap_or(false);
		let create_tracker_with_index = json_value["create tracker with index"].as_i32().unwrap_or(0);
		let label_image_index = json_value["label image index"].as_i32().unwrap_or(0);
		let current_slam_keyframe_ui_state = SlamKeyframeInspectState::from_json_or_default(&json_value["current slam keyframe ui state"]);
		
		Self {
			create_tracker_with_index,
			create_tracker_on_click,
			label_image_index,
			current_slam_keyframe_ui_state
		}
	}
}
