pub mod keyframe;
pub mod pixel_tracker;
pub mod camera;

use keyframe::SlamKeyframe;
use std::rc::Rc;

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
	
	
}


