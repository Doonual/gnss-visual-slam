pub mod keyframe;
pub mod pixel_tracker;
pub mod camera;

use glam::{Vec2, Vec3, Quat, Mat4, EulerRot};

use keyframe::SlamKeyframe;
use std::rc::Rc;
use crate::ui::scene::DrawSceneGizmo;
use crate::ui::scene::Scene3D;

use easy_imgui as imgui;

pub struct SlamState {
	pub slam_keyframes: Vec<SlamKeyframe>
}

impl SlamState {
	
	pub fn init(gl: Rc<glow::Context>) -> Self {
		
		
		let image_paths = [
			"images/1.jpg",
			"images/2.jpg",
			"images/3.jpg",
			"images/4.jpg",
			"images/5.jpg",
		];
		let mut opened_images = Vec::new();
		for image_path in image_paths {
			match SlamKeyframe::create_from_file(gl.clone(), image_path) {
				Ok(image) => opened_images.push(image),
				Err(e) => println!("Failed to open image: {e}"),
			}
		}
		
		SlamState {
			slam_keyframes: opened_images,
		}
		
	}
	
	pub fn get_total_loss(&mut self) -> f32 {
		
		let mut num_pairs_found = 0;
		let mut distance_sum = 0.0;
		
		for i in 0..self.slam_keyframes.len() {
			if let Some(keyframe_a) = &self.slam_keyframes.get(i) {
				
				for n in (i+1)..self.slam_keyframes.len() {
					if i == n {continue;}
					if let Some(keyframe_b) = &self.slam_keyframes.get(n) {
						for keyframe_a_tracker in &keyframe_a.trackers {
							for keyframe_b_tracker in &keyframe_b.trackers {
								if keyframe_a_tracker.point_index != keyframe_b_tracker.point_index {continue;}
								
								if let Some((dist, pos_a, pos_b)) = shortest_dist_camera_clamped(keyframe_a, keyframe_a_tracker, keyframe_b, keyframe_b_tracker) {
									distance_sum += dist;
									num_pairs_found += 1;
									
									//scene.line(pos_a, pos_b, 255, 255, 0, 255);
									
								}
								
							}
						}
					}
					
					
				}
			}
			
		}
		
		
		let average_distance = distance_sum / num_pairs_found as f32;
		average_distance
	}
	
	pub fn optimise_keyframe(&mut self, keyframe: i32, learn_rate: f32) -> Option<bool> {
		
		
		
		let starting_loss = self.get_total_loss();
		let delta = 0.0001;
		
		let initial_pos = self.slam_keyframes[keyframe as usize].slam_camera.position;
		let initial_rot = self.slam_keyframes[keyframe as usize].slam_camera.rotation;
		
		self.slam_keyframes[keyframe as usize].slam_camera.position = initial_pos + Vec3::X * delta;
		let loss_delta_x = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.position = initial_pos + Vec3::Y * delta;
		let loss_delta_y = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.position = initial_pos + Vec3::Z * delta;
		let loss_delta_z = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.position = initial_pos;
		
		self.slam_keyframes[keyframe as usize].slam_camera.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, delta, 0.0, 0.0);
		let loss_delta_pitch = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, 0.0, delta, 0.0);
		let loss_delta_yaw = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, delta);
		let loss_delta_roll = (self.get_total_loss() - starting_loss) / delta;
		
		self.slam_keyframes[keyframe as usize].slam_camera.position = initial_pos - Vec3{x: loss_delta_x, y: loss_delta_y, z: loss_delta_z} * learn_rate;
		self.slam_keyframes[keyframe as usize].slam_camera.rotation = initial_rot * Quat::from_euler(EulerRot::XYZ, -loss_delta_pitch * learn_rate, -loss_delta_yaw * learn_rate, -loss_delta_roll * learn_rate);
		
		Some(true)
		
	}
	
	
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
	
	let line_a_point = keyframe_a.slam_camera.position;
	let line_a_vec = keyframe_a.slam_camera.project_tracker(tracker_a);
	
	let line_b_point = keyframe_b.slam_camera.position;
	let line_b_vec = keyframe_b.slam_camera.project_tracker(tracker_b);
	
	let ((dist, point_on_a, point_on_b)) = shortest_dist(line_a_point, line_a_vec, line_b_point, line_b_vec)?;
	
	
	
	// Must clamp the close points to the points beyond the camera's
	// sesor plane
	
	let a_pos_local = keyframe_a.slam_camera.rotation.inverse().mul_vec3(point_on_a - line_a_point);
	let a_vec_local = keyframe_a.slam_camera.rotation.inverse().mul_vec3(line_a_vec);
	
	let b_pos_local = keyframe_b.slam_camera.rotation.inverse().mul_vec3(point_on_b - line_b_point);
	let b_vec_local = keyframe_b.slam_camera.rotation.inverse().mul_vec3(line_b_vec);
	
	let collision_dist_a = a_pos_local.z / a_vec_local.z;
	let min_collision_dist_a = keyframe_a.slam_camera.get_view_dist() / a_vec_local.z;
	
	let collision_dist_b = b_pos_local.z / b_vec_local.z;
	let min_collision_dist_b = keyframe_b.slam_camera.get_view_dist() / b_vec_local.z;
	
	let new_a_pos_local = keyframe_a.slam_camera.position + keyframe_a.slam_camera.rotation.mul_vec3(a_vec_local * f32::max(min_collision_dist_a, collision_dist_a));
	let new_b_pos_local = keyframe_b.slam_camera.position + keyframe_b.slam_camera.rotation.mul_vec3(b_vec_local * f32::max(min_collision_dist_b, collision_dist_b));
	
	Some((new_a_pos_local.distance(new_b_pos_local), new_a_pos_local, new_b_pos_local))
	
	
}


impl DrawSceneGizmo for SlamState {
	fn draw_scene_gizmo(&mut self, scene: &mut Scene3D) {
		
		
		let dim = 5;
		let scale = 10.0;
		for x in -dim..dim+1 {
			let val = (x as f32) / dim as f32;
			
			scene.line(glam::vec3(val, 0.0, -1.0) * scale, glam::vec3(val, 0.0, 1.0) * scale, 200, 200, 200, 80);
			scene.line(glam::vec3(-1.0, 0.0, val) * scale, glam::vec3(1.0, 0.0, val) * scale, 200, 200, 200, 80);
		}
		
		for slam_keyrame in &mut self.slam_keyframes {
			slam_keyrame.draw_scene_gizmo(scene);
		}
		
		let mut num_pairs_found = 0;
		let mut distance_sum = 0.0;
		
		for i in 0..self.slam_keyframes.len() {
			if let Some(keyframe_a) = &self.slam_keyframes.get(i) {
				
				for n in (i+1)..self.slam_keyframes.len() {
					if i == n {continue;}
					if let Some(keyframe_b) = &self.slam_keyframes.get(n) {
						for keyframe_a_tracker in &keyframe_a.trackers {
							for keyframe_b_tracker in &keyframe_b.trackers {
								if keyframe_a_tracker.point_index != keyframe_b_tracker.point_index {continue;}
								
								if let Some((dist, pos_a, pos_b)) = shortest_dist_camera_clamped(keyframe_a, keyframe_a_tracker, keyframe_b, keyframe_b_tracker) {
									distance_sum += dist;
									num_pairs_found += 1;
									
									scene.line(pos_a, pos_b, 255, 255, 0, 255);
									
								}
								
							}
						}
					}
					
					
				}
			}
			
		}
		
		
		let average_distance = distance_sum / num_pairs_found as f32;
		println!("Num pairs: {num_pairs_found} | Average distance: {average_distance}");
		
	}
}
