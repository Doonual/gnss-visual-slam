use std::fmt;



use glow;
use glow::HasContext;
use std::rc::Rc;

use crate::slam::pixel_tracker;

pub struct SlamKeyframe {
	
	pub image: Option<glow::Texture>,
	pub image_width: u32,
	pub image_height: u32,
	
	pub trackers: Vec<pixel_tracker::PixelTracker>
	
}

pub enum CreateSlamKeyframeError {
	ImageOpenError(image::ImageError),
	CreateTextureError(String),
}



impl fmt::Display for CreateSlamKeyframeError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CreateSlamKeyframeError::ImageOpenError(image_error) => {
				return write!(fmt, "Image open error: {image_error}");
			},
			CreateSlamKeyframeError::CreateTextureError(error_string) => {
				return write!(fmt, "Error creating glow texture: {error_string}");
			}
		}
	}
}



impl SlamKeyframe {
	
	pub fn create_from_file(gl: Rc<glow::Context>, path: &str) -> Result<Self, CreateSlamKeyframeError> {
		
		
		let image_reader = match image::open(path) {
			Err(e) => return Err(CreateSlamKeyframeError::ImageOpenError(e)),
			Ok(r) => r,
		};
		
		
		let image = image_reader.flipv().to_rgba8();
		let (w, h) = image.dimensions();
		
		unsafe {
			
			let tex = match gl.create_texture() {
				Err(e) => return Err(CreateSlamKeyframeError::CreateTextureError(e)),
				Ok(r) => r,
			};
			
			
			gl.bind_texture(glow::TEXTURE_2D, Some(tex));
			
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
			
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA8 as i32,
				w as i32,
				h as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(image.as_raw())),
			);
			
			
			Ok(SlamKeyframe {
				image: Some(tex),
				image_width: w,
				image_height: h,
				
				trackers: Vec::new(),
				
			})
			
			
		}
		
		
		
	}
	
	
}


