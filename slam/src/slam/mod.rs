pub mod keyframe;
pub mod pixel_tracker;
//pub mod camera;

use std::{collections::HashMap, hash::DefaultHasher};

use glam::{Vec3, Quat, EulerRot};
use json::JsonValue;



use crate::{file::{self, Saveable}, scene::{DrawSceneGizmo, Scene3D}, slam::keyframe::{SlamKeyframe, SlamKeyframeGizmoState}};


#[derive(Default)]
pub struct SlamState {
	pub slam_keyframes: Vec<SlamKeyframe>,
	pub enabled_keyframes: HashMap<u64, bool>,
}

impl SlamState {
	
	pub fn calculate_track_point_position(&mut self, point_index: i32) -> Option<Vec3> {
		
		//let potential_positions : Vec<Vec3> = Vec::new();
		let mut average_pos = Vec3::ZERO;
		let mut num_sum = 0;
		
		for i in 0..self.slam_keyframes.len() {
			
			let keyframe_a_enabled = true;
			if let Some(keyframe_a) = &self.slam_keyframes.get(i) && keyframe_a_enabled {
				
				for n in (i+1)..self.slam_keyframes.len() {
					if i == n {continue;}
					
					let keyframe_b_enabled = true;
					if let Some(keyframe_b) = &self.slam_keyframes.get(n) && keyframe_b_enabled {
						
						for keyframe_a_tracker in &keyframe_a.trackers {
							for keyframe_b_tracker in &keyframe_b.trackers {
								
								if keyframe_a_tracker.point_index != point_index || keyframe_b_tracker.point_index != point_index { continue; }
								
								if let Some((_dist_, pos_a, pos_b)) = shortest_dist_camera_clamped(keyframe_a, keyframe_a_tracker, keyframe_b, keyframe_b_tracker) {
									average_pos += (pos_a + pos_b) / 2.0;
									num_sum += 1;
								};
								
								
							}
						}
						
						
					}
					
				}
			}
		}
		
		if num_sum == 0 {
			return None;
		}
		Some(average_pos / num_sum as f32)
		
		
	}
	
	pub fn calculate_loss(&self, loss_method: &SlamKeyframeLossMethod) -> f32 {
		
		
		let mut loss = 0.0;
		let mut loss_count = 0;
		
		for i in 0..self.slam_keyframes.len() {
			if let Some(slam_keyframe) = &self.slam_keyframes.get(i) && let Some(keyframe_enabled) = self.enabled_keyframes.get(&slam_keyframe.get_hash_value()) && *keyframe_enabled {
				
				loss += self.calculate_keyframe_loss(i, loss_method).unwrap_or(0.0);
				loss_count += 1;
			}
		}
		
		if loss_count == 0 {
			return 0.0
		}
		
		loss / loss_count as f32
		
	}
	
	pub fn calculate_keyframe_loss(&self, keyframe_index: usize, loss_method: &SlamKeyframeLossMethod) -> Option<f32> {
		match loss_method {
			SlamKeyframeLossMethod::AgainstOtherKeyframe { other_keyframe_index, distance_from_camera_compensation } => {
				
				let mut loss_sum = 0.0;
				let mut loss_sum_count = 0;
				
				let keyframe = &self.slam_keyframes.get(keyframe_index)?;
				for tracker in &keyframe.trackers {
					
					let mut other_keyframe_enabled = false;
					if let Some(keyframe_enabled) = self.enabled_keyframes.get(&keyframe.get_hash_value()) && *keyframe_enabled {
						other_keyframe_enabled = true;
					}
					if !other_keyframe_enabled {continue;}
					
					let other_keyframe = &self.slam_keyframes.get(*other_keyframe_index)?;
					for other_tracker in &other_keyframe.trackers {
						if other_tracker.point_index != tracker.point_index {continue;}
						let (dist, pos_a, _) = shortest_dist_camera_clamped(keyframe, tracker, other_keyframe, other_tracker)?;
						
						match distance_from_camera_compensation {
							true => {
								let distance_normalised_dist = dist / keyframe.camera_pose.position.distance(pos_a);
								loss_sum += f32::powi(distance_normalised_dist, 6);
								loss_sum_count += 1;
							},
							false => {
								loss_sum += f32::powi(dist, 2);
								loss_sum_count += 1;
							}
						}
					}
					
				}
				
				if loss_sum_count == 0 {return None}
				Some(loss_sum / loss_sum_count as f32)
				
				
			},
			SlamKeyframeLossMethod::AgainstAllKeyframes {distance_from_camera_compensation} => {
				
				let mut loss = 0.0;
				let mut loss_count = 0;
				for i in 0..self.slam_keyframes.len() {
					if i == keyframe_index {continue;}
					if let Some(current_loss) = self.calculate_keyframe_loss(keyframe_index, &SlamKeyframeLossMethod::AgainstOtherKeyframe{other_keyframe_index: i, distance_from_camera_compensation: *distance_from_camera_compensation}){
						loss += f32::powi(current_loss, 2);
						loss_count += 1;
					}
				}
				if loss_count == 0 {return None}
				Some(loss / loss_count as f32)
				
			},
		}
	}
	
	pub fn optimise_keyframe(&mut self, keyframe: usize, loss_method: &SlamKeyframeLossMethod, learn_rate: f32) {
		
		
		
		let mut this_keyframe_enabled = false;
		if let Some(keyframe) = self.slam_keyframes.get(keyframe) && let Some(this_keyframe_enabled_actual) = self.enabled_keyframes.get(&keyframe.get_hash_value()) && *this_keyframe_enabled_actual {
			this_keyframe_enabled = true;
		}
		if !this_keyframe_enabled { return; }
		
		let starting_loss = self.calculate_loss(loss_method);
		let delta = 0.00001;
		
		let initial_pos = self.slam_keyframes[keyframe].camera_pose.position;
		let initial_rot = self.slam_keyframes[keyframe].camera_pose.rotation;
		
		self.slam_keyframes[keyframe].camera_pose.position = initial_pos + Vec3::X * delta;
		let loss_delta_x = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.position = initial_pos + Vec3::Y * delta;
		let loss_delta_y = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.position = initial_pos + Vec3::Z * delta;
		let loss_delta_z = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.position = initial_pos;
		
		self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, delta, 0.0, 0.0);
		let loss_delta_pitch = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, 0.0, delta, 0.0);
		let loss_delta_yaw = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, delta);
		let loss_delta_roll = (self.calculate_loss(loss_method) - starting_loss) / delta;
		
		self.slam_keyframes[keyframe].camera_pose.position = initial_pos - Vec3{x: loss_delta_x, y: loss_delta_y, z: loss_delta_z} * learn_rate;
		self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, -loss_delta_pitch * learn_rate, -loss_delta_yaw * learn_rate, -loss_delta_roll * learn_rate);
		
		
		// Check this actually reduced the loss
		let current_loss = self.calculate_loss(loss_method);
		if current_loss >= starting_loss {
			self.slam_keyframes[keyframe].camera_pose.position = initial_pos;
			self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot;
		}
		
		
		
	}
	
	pub fn jump_optimise_keyframe(&mut self, keyframe: usize, loss_method: &SlamKeyframeLossMethod) {
		
		let mut this_keyframe_enabled = false;
		if let Some(keyframe) = self.slam_keyframes.get(keyframe) && let Some(this_keyframe_enabled_actual) = self.enabled_keyframes.get(&keyframe.get_hash_value()) && *this_keyframe_enabled_actual {
			this_keyframe_enabled = true;
		}
		if !this_keyframe_enabled { return; }
		
		
		let initial_loss = self.calculate_loss(loss_method);
		
		let initial_pos = self.slam_keyframes[keyframe].camera_pose.position;
		let initial_rot = self.slam_keyframes[keyframe].camera_pose.rotation;
		
		let exponent = rand::random::<f32>() * 3.0 - 1.0;
		let random_sign = match rand::random::<bool>() {
			true => 1.0,
			false => -1.0,
		};
		let random_value = f32::powf(10.0, exponent) * random_sign;
		
		self.slam_keyframes[keyframe].camera_pose.position.x += random_value;
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.position = initial_pos;
		}
		else {
			println!("Random move worked: x: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		self.slam_keyframes[keyframe].camera_pose.position.y += random_value;
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.position = initial_pos;
		}
		else {
			println!("Random move worked: y: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		self.slam_keyframes[keyframe].camera_pose.position.z += random_value;
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.position = initial_pos;
		}
		else {
			println!("Random move worked: z: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		self.slam_keyframes[keyframe].camera_pose.rotation *= Quat::from_euler(EulerRot::XYZ, random_value, 0.0, 0.0);
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot;
		}
		else {
			println!("Random rotate worked: pitch: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		self.slam_keyframes[keyframe].camera_pose.rotation *= Quat::from_euler(EulerRot::XYZ, 0.0, random_value, 0.0);
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot;
		}
		else {
			println!("Random rotate worked: yaw: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		self.slam_keyframes[keyframe].camera_pose.rotation *= Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, random_value);
		if self.calculate_loss(loss_method) > initial_loss {
			self.slam_keyframes[keyframe].camera_pose.rotation = initial_rot;
		}
		else {
			println!("Random rotate worked: roll: {random_value}. Initial loss: {initial_loss}, new loss: {}", self.calculate_loss(loss_method));
		}
		
		
	}
	
	pub fn optimise_all_keyframes(&mut self, learn_rate: f32) {
		
		let loss_method = SlamKeyframeLossMethod::AgainstAllKeyframes{distance_from_camera_compensation: false};
		let test_loss_method = SlamKeyframeLossMethod::AgainstOtherKeyframe{other_keyframe_index: (rand::random::<u16>() % self.slam_keyframes.len() as u16).into(), distance_from_camera_compensation: true};
		
		for i in 1..self.slam_keyframes.len() {
			
			let mut this_keyframe_enabled = false;
			if let Some(keyframe) = self.slam_keyframes.get(i) && let Some(this_keyframe_enabled_actual) = self.enabled_keyframes.get(&keyframe.get_hash_value()) && *this_keyframe_enabled_actual {
				this_keyframe_enabled = true;
			}
			if !this_keyframe_enabled {continue;}
			
			//self.jump_optimise_keyframe(i, &loss_method);
			self.optimise_keyframe(i, &loss_method, learn_rate);
			
		}
		
		println!("Loss: {}", self.calculate_loss(&loss_method));
		
	}
	
	pub fn create_missing_enabled_keyframe_entries(&mut self) {
		
		for slam_keyframe in &self.slam_keyframes {
			
			let slam_keyframe_hash = slam_keyframe.get_hash_value();
			if !self.enabled_keyframes.contains_key(&slam_keyframe_hash) {
				self.enabled_keyframes.insert(slam_keyframe_hash, true);
			}
			
		}
		
	}
}

impl DrawSceneGizmo for SlamState {
	
	type GizmoState = SlamStateGizmoState;
	
	fn draw_scene_gizmo(&mut self, gizmo_state: &mut Self::GizmoState, scene: &mut Scene3D) {
		
		let dim = 5;
		let scale = 10.0;
		for x in -dim..dim+1 {
			let val = (x as f32) / dim as f32;
			
			scene.line(glam::vec3(val, 0.0, -1.0) * scale, glam::vec3(val, 0.0, 1.0) * scale, [200, 200, 200, 80]);
			scene.line(glam::vec3(-1.0, 0.0, val) * scale, glam::vec3(1.0, 0.0, val) * scale, [200, 200, 200, 80]);
		}
		
		for slam_keyrame in &mut self.slam_keyframes {
			if let Some(keyframe_enabled) = self.enabled_keyframes.get(&slam_keyrame.get_hash_value()) && *keyframe_enabled {
				slam_keyrame.draw_scene_gizmo(&mut SlamKeyframeGizmoState::default(), scene);
			}
		}
		
		let mut done = false;
		let mut point_index = 0;
		let tracker_size = 0.01;
		while !done {
			
			if let Some(track_pos) = self.calculate_track_point_position(point_index) {
				
				scene.sphere(track_pos, Quat::IDENTITY, tracker_size, [0, 255, 0, 255]);
				
				point_index += 1;
				
			}
			else {
				done = true;
			}
			
			
		}
		
	}
	
}

impl file::Saveable for SlamState {
	
	fn to_json(&self) -> json::JsonValue {
		
		let mut main_json = json::JsonValue::new_object();
		
		let mut keyframe_array_json = json::JsonValue::new_array();
		for slam_keyframe in &self.slam_keyframes {
			if let Err(e) = keyframe_array_json.push(slam_keyframe.to_json()) {
				eprintln!("Failed to push to vec: {e}");
			}
		}
		
		main_json["keyframes"] = keyframe_array_json;
		
		main_json
		
	}
	
	fn from_json(json_value: &JsonValue) -> Result<SlamState, file::DecodeJsonError> {
		
		let keyframe_array_json = &json_value["keyframes"];
		
		let mut keyframes = Vec::new();
		
		for i in 0..keyframe_array_json.len() {
			let keyframe_json = &keyframe_array_json[i];
			keyframes.push(SlamKeyframe::from_json(&keyframe_json)?);
		}
		keyframes.reverse();
		
		let mut enabled_keyframes: HashMap<u64, bool> = HashMap::new();
		for keyframe in &keyframes {
			enabled_keyframes.insert(keyframe.get_hash_value(), true);
		}
		
		Ok(SlamState {
			slam_keyframes: keyframes,
			enabled_keyframes,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let keyframe_array_json = &json_value["keyframes"];
		
		let mut keyframes = Vec::new();
		
		for i in 0..keyframe_array_json.len() {
			let keyframe_json = &keyframe_array_json[i];
			keyframes.push(SlamKeyframe::from_json_or_default(&keyframe_json));
		}
		keyframes.reverse();
		
		let mut enabled_keyframes: HashMap<u64, bool> = HashMap::new();
		for keyframe in &keyframes {
			enabled_keyframes.insert(keyframe.get_hash_value(), true);
		}
		
		SlamState {
			slam_keyframes: keyframes,
			enabled_keyframes,
		}
	}
	
}

#[derive(Default)]
pub struct SlamStateGizmoState {
}

impl Saveable for SlamStateGizmoState {
	fn to_json(&self) -> json::JsonValue {
		JsonValue::new_object()
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, file::DecodeJsonError> {
		Ok(Self::default())
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		Self::default()
	}
}

pub enum SlamKeyframeLossMethod {
	AgainstOtherKeyframe{
		other_keyframe_index: usize,
		distance_from_camera_compensation: bool,
	},
	AgainstAllKeyframes {
		distance_from_camera_compensation: bool,
	},
}


// Returns (distance, closest point on a, closest point on b). None if parallel
pub fn shortest_dist(line_a_point: Vec3, line_a_vec: Vec3, line_b_point: Vec3, line_b_vec: Vec3) -> Option<(f32, Vec3, Vec3)> {
	
	let w = line_a_point - line_b_point;
	let a = line_a_vec.dot(line_a_vec);
	let b = line_a_vec.dot(line_b_vec);
	let c = line_b_vec.dot(line_b_vec);
	let d = line_a_vec.dot(w);
	let e = line_b_vec.dot(w);
	
	let denom = a * c - b * b;
	if denom.abs() < 1e-8 {
		return None;
	}
	
	let s = (b * e - c * d) / denom;
	let t = (a * e - b * d) / denom;
	
	let q1 = line_a_point + line_a_vec * s;
	let q2 = line_b_point + line_b_vec * t;
	Some((q1.distance(q2), q1, q2))
	
}

pub fn shortest_dist_camera_clamped(keyframe_a: &SlamKeyframe, tracker_a: &pixel_tracker::PixelTracker, keyframe_b: &SlamKeyframe, tracker_b: &pixel_tracker::PixelTracker) -> Option<(f32, Vec3, Vec3)> {
	
	let line_a_point = keyframe_a.camera_pose.position;
	let line_a_vec = keyframe_a.project_tracker(tracker_a);
	
	
	let line_b_point = keyframe_b.camera_pose.position;
	let line_b_vec = keyframe_b.project_tracker(tracker_b);
	
	let (_, point_on_a, point_on_b) = shortest_dist(line_a_point, line_a_vec, line_b_point, line_b_vec)?;
	
	
	
	// Must clamp the close points to the points beyond the camera's
	// sesor plane
	
	let a_pos_local = keyframe_a.camera_pose.rotation.inverse().mul_vec3(point_on_a - line_a_point);
	let a_vec_local = keyframe_a.camera_pose.rotation.inverse().mul_vec3(line_a_vec);
	
	let b_pos_local = keyframe_b.camera_pose.rotation.inverse().mul_vec3(point_on_b - line_b_point);
	let b_vec_local = keyframe_b.camera_pose.rotation.inverse().mul_vec3(line_b_vec);
	
	let collision_dist_a = a_pos_local.z / a_vec_local.z;
	let min_collision_dist_a = keyframe_a.get_view_dist() / a_vec_local.z;
	
	let collision_dist_b = b_pos_local.z / b_vec_local.z;
	let min_collision_dist_b = keyframe_b.get_view_dist() / b_vec_local.z;
	
	let new_a_pos_local = keyframe_a.camera_pose.position + keyframe_a.camera_pose.rotation.mul_vec3(a_vec_local * f32::max(min_collision_dist_a, collision_dist_a));
	let new_b_pos_local = keyframe_b.camera_pose.position + keyframe_b.camera_pose.rotation.mul_vec3(b_vec_local * f32::max(min_collision_dist_b, collision_dist_b));
	
	Some((new_a_pos_local.distance(new_b_pos_local), new_a_pos_local, new_b_pos_local))
	
	
}





