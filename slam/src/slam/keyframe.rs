use easy_imgui as imgui;
use easy_imgui::Vector2;
use std::fmt;


use easy_imgui::TextureRef;
use easy_imgui_renderer::Renderer;

use glow;
use glow::HasContext;
use std::rc::Rc;

pub struct SlamKeyframe {
	
	image: Option<glow::Texture>,
	image_width: u32,
	image_height: u32,
	
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
			})
			
			
		}
		
		
		
	}
	
	pub fn imgui_inspect(&self, imgui: &imgui::Ui<crate::App>) {
		
		
		
		match self.image {
			Some(texture) => {
				let tex_id = Renderer::map_tex(texture);
				let tex_ref = TextureRef::Id(tex_id);
				
				
				let mut width = imgui.get_content_region_avail().x;
				if width > 400.0 {
					width = 400.0;
				}
				let height = width * self.image_height as f32 / self.image_width as f32;
				
				let border_width = 2.0;
				
				
				imgui.with_push((imgui::StyleVar::FramePadding, imgui::StyleValue::Vec2(Vector2{x: 0.0, y: 0.0})), || {
					imgui.with_push((imgui::StyleVar::FrameBorderSize, imgui::StyleValue::F32(border_width)), || {
						imgui.image_button_config("test image".into(), tex_ref, Vector2{x: width, y: height}).uv0(Vector2{x: 0.0, y: 1.0}).uv1(Vector2{x: 1.0, y: 0.0}).build();
					});
				});
				
			},
			None => {
				imgui.text("Texure is fucked");
			}
		}
		
	}
	
}


