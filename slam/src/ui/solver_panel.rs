use easy_imgui as imgui;
use json::JsonValue;
use crate::{file::{DecodeJsonError, Saveable}, slam::{SlamKeyframeLossMethod, SlamState}, ui::ImguiInspect};


#[derive(Default)]
pub struct SolverPanel {
	
	optimise_rate: f32,
	optimise_on: bool,
	
	
	loss_plot: SimplePlot,
	recent_loss_plot: RollingPlot,
	
}


impl SolverPanel {
	
	pub fn draw_imgui_panel(&mut self, slam_state: &mut SlamState, imgui: &easy_imgui::Ui<crate::App>) {
		
		imgui.slider_float_config("Optimise rate".into(), &mut self.optimise_rate).flags(imgui::SliderFlags::Logarithmic).range(0.001, 100.0).build();
		
		imgui.checkbox("Optimise camera".into(), &mut self.optimise_on);
		if self.optimise_on {
			slam_state.optimise_all_keyframes(self.optimise_rate);
			
			let current_loss = slam_state.calculate_loss(&SlamKeyframeLossMethod::AgainstAllKeyframes { distance_from_camera_compensation: true});
			let plot_val = f32::log10(current_loss);
			
			
			self.loss_plot.add_point(plot_val);
			self.recent_loss_plot.add_point(plot_val);
		}
		
		
		
		self.recent_loss_plot.scale_max = self.recent_loss_plot.get_max_value().unwrap_or(1.0) + 0.5;
		self.recent_loss_plot.scale_min = self.recent_loss_plot.get_min_value().unwrap_or(-2.0) - 0.5;
		//self.recent_loss_plot.draw_imgui_plot(imgui);
		
		self.loss_plot.scale_max = self.loss_plot.get_max_value().unwrap_or(1.0) + 0.5;
		self.loss_plot.scale_min = -2.5;
		self.loss_plot.draw_imgui_plot(imgui);
		
		
	}
	
	
}

impl Saveable for SolverPanel {
	fn to_json(&self) -> json::JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		json_obj["optimise rate"] = self.optimise_rate.into();
		json_obj["optimise on"] = self.optimise_on.into();
		json_obj["loss plot"] = self.loss_plot.to_json();
		json_obj["recent loss plot"] = self.recent_loss_plot.to_json();
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, crate::file::DecodeJsonError> {
		let Some(optimise_rate) = json_value["optimise rate"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "optimise rate".into() }); };
		let Some(optimise_on) = json_value["optimise on"].as_bool() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "optimise on".into() }); };
		let loss_plot = SimplePlot::from_json(&json_value["loss plot"])?;
		let recent_loss_plot = RollingPlot::from_json(&json_value["recent loss plot"])?;
		
		Ok(Self {
			optimise_rate,
			optimise_on,
			loss_plot,
			recent_loss_plot,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let optimise_rate = json_value["optimise rate"].as_f32().unwrap_or(0.01);
		let optimise_on = json_value["optimise on"].as_bool().unwrap_or(false);
		let loss_plot = SimplePlot::from_json_or_default(&json_value["loss plot"]);
		let recent_loss_plot = RollingPlot::from_json_or_default(&json_value["recent loss plot"]);
		
		Self {
			optimise_rate,
			optimise_on,
			loss_plot,
			recent_loss_plot,
		}
	}
}


pub trait Plot {
	
	fn values(&self) -> Vec<f32>;
	
	fn get_max_value(&self) -> Option<f32> {
		let mut max = None;
		let values = self.values();
		for value in values {
			if value > max.unwrap_or(f32::MIN) { max = Some(value); }
		}
		max
	}
	fn get_min_value(&self) -> Option<f32> {
		let mut min = None;
		let values = self.values();
		for value in values {
			if value < min.unwrap_or(f32::MAX) { min = Some(value); }
		}
		min
	}
	
	fn get_plot_bounds_min(&self) -> f32;
	fn get_plot_bounds_max(&self) -> f32;
	
	fn draw_imgui_plot(&self, imgui: &imgui::Ui<crate::App>) {
		
		let window_avail = imgui.get_content_region_avail();
		
		imgui.child_config("##simpleplot".into()).size(imgui::vec2(window_avail.x, window_avail.x)).child_flags(imgui::ChildFlags::Borders).with(|| {
			
			let top_left = imgui.get_cursor_screen_pos();
			let avail = imgui.get_content_region_avail();
			
			let values = self.values();
			let min = self.get_plot_bounds_min();
			let max = self.get_plot_bounds_max();
			
			for i in 0..(values.len() - 1) {
				
				let uv_x_start = i as f32 / (values.len() - 1) as f32;
				let uv_x_end = (i + 1) as f32 / (values.len() - 1) as f32;
				
				let uv_y_start = 1.0 - (values[i] - min) / (max - min);
				let uv_y_end = 1.0 - (values[i + 1] - min) / (max - min);
				
				let window_list = imgui.window_draw_list();
				
				window_list.add_line(imgui::vec2(top_left.x + uv_x_start * avail.x, top_left.y + uv_y_start * avail.y), imgui::vec2(top_left.x + uv_x_end * avail.x, top_left.y + uv_y_end * avail.y), imgui::Color::RED, 1.0);
				
				
			}
			
			
		});
		
	}
	
}





struct SimplePlot {
	
	pub samples:Vec<f32>,
	pub scale_min: f32,
	pub scale_max: f32,
	
}

impl SimplePlot {
	fn add_point(&mut self, x: f32) {
		self.samples.push(x);
	}
	
}

impl Default for SimplePlot {
	
	fn default() -> Self {
		
		Self {
			samples: Vec::<f32>::new(),
			scale_min: 0.0,
			scale_max: 1.0,
		}
		
	}
	
}

impl Plot for SimplePlot {
	fn values(&self) -> Vec<f32> {
		self.samples.clone()
	}
	
	fn get_plot_bounds_min(&self) -> f32 {
		self.scale_min
	}
	
	fn get_plot_bounds_max(&self) -> f32 {
		self.scale_max
	}
}

impl Saveable for SimplePlot {
	fn to_json(&self) -> JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		
		let mut samples_arr = JsonValue::new_array();
		for sample in &self.samples {
			samples_arr.push(*sample);
		}
		json_obj["samples"] = samples_arr;
		
		json_obj["scale min"] = self.scale_min.into();
		json_obj["scale max"] = self.scale_max.into();
		
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let samples_arr = &json_value["samples"];
		let mut samples_vec = Vec::<f32>::new();
		
		for i in 0..samples_arr.len() {
			if let Some(current_sample) = samples_arr[i].as_f32() {
				samples_vec.push(current_sample);
			}
		}
		
		
		let Some(scale_min) = json_value["scale min"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "scale min".into() }); };
		let Some(scale_max) = json_value["scale max"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "scale max".into() }); };
		
		Ok(Self {
			samples: samples_vec,
			scale_min,
			scale_max,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		
		let samples_arr = &json_value["samples"];
		let mut samples_vec = Vec::<f32>::new();
		
		for i in 0..samples_arr.len() {
			if let Some(current_sample) = samples_arr[i].as_f32() {
				samples_vec.push(current_sample);
			}
		}
		
		
		let scale_min = json_value["scale min"].as_f32().unwrap_or(0.0);
		let scale_max = json_value["scale max"].as_f32().unwrap_or(0.0);
		
		Self {
			samples: samples_vec,
			scale_min,
			scale_max,
		}
	}
}




struct RollingPlot {
	
	num_samples: usize,
	write_pos: usize,
	pub samples: Vec<f32>,
	
	pub scale_min: f32,
	pub scale_max: f32,
	
}

impl RollingPlot {
	
	fn new(num_samples: usize) -> Self {
		
		let mut x_vec = Vec::new();
		for i in 0..num_samples {
			x_vec.push(0.0);
		}
		
		RollingPlot {
			num_samples,
			write_pos: 0,
			samples: x_vec,
			
			scale_min: 1.0,
			scale_max: 0.0,
		}
		
	}
	
	fn add_point(&mut self, x: f32) {
		self.samples[self.write_pos] = x;
		self.write_pos += 1;
		self.write_pos %= self.num_samples;
	}
	
}

impl Default for RollingPlot {
	fn default() -> Self {
		Self::new(1024)
	}
}

impl Plot for RollingPlot {
	
	fn values(&self) -> Vec<f32> {
		let (first_half,second_half) = self.samples.split_at(self.write_pos);
		let rearranged = [second_half, first_half].concat().to_vec();
		rearranged
	}
	fn get_plot_bounds_min(&self) -> f32 {
		self.scale_min
	}
	fn get_plot_bounds_max(&self) -> f32 {
		self.scale_max
	}
	
}

impl Saveable for RollingPlot {
	fn to_json(&self) -> JsonValue {
		
		let mut json_obj = JsonValue::new_object();
		json_obj["num samples"] = self.num_samples.into();
		json_obj["write pos"] = self.write_pos.into();
		
		let mut samples_arr = JsonValue::new_array();
		for sample in &self.samples {
			samples_arr.push(*sample);
		}
		json_obj["samples"] = samples_arr;
		
		json_obj["scale min"] = self.scale_min.into();
		json_obj["scale max"] = self.scale_max.into();
		
		json_obj
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(num_samples) = json_value["num samples"].as_usize() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "num samples".into() }); };
		let Some(write_pos) = json_value["write pos"].as_usize() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "write pos".into() }); };
		
		let mut samples_arr = &json_value["samples"];
		let mut samples_vec = Vec::<f32>::new();
		
		for i in 0..samples_arr.len() {
			if let Some(current_sample) = samples_arr[i].as_f32() {
				samples_vec.push(current_sample);
			}
		}
		
		
		let Some(scale_min) = json_value["scale min"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "scale min".into() }); };
		let Some(scale_max) = json_value["scale max"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "scale max".into() }); };
		
		Ok(Self {
			num_samples,
			write_pos,
			
			samples: samples_vec,
			
			scale_min,
			scale_max,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let Some(num_samples) = json_value["num samples"].as_usize() else {return Self::default();};
		let Some(write_pos) = json_value["write pos"].as_usize() else {return Self::default();};
		
		let samples_arr = &json_value["samples"];
		let mut samples_vec = Vec::<f32>::new();
		
		for i in 0..samples_arr.len() {
			if let Some(current_sample) = samples_arr[i].as_f32() {
				samples_vec.push(current_sample);
			}
		}
		
		
		let scale_min = json_value["scale min"].as_f32().unwrap_or(0.0);
		let scale_max = json_value["scale max"].as_f32().unwrap_or(1.0);
		
		Self {
			num_samples,
			write_pos,
			
			samples: samples_vec,
			
			scale_min,
			scale_max,
		}
	}
}



