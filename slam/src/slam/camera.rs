use glam;

pub struct SlamCamera {
	view_matrix: glam::Mat4,
	pub position: glam::Vec3,
	pub rotation: glam::Quat,
}

impl Default for SlamCamera {
	fn default() -> Self {
		
		let iphone14_view_matrix = [
			0.7515, 0.0000, 0.5000, 0.0000,
			0.0000, 1.0020, 0.5000, 0.0000,
			0.0000, 0.0000, 1.0000, 0.0000,
			0.0000, 0.0000, 0.0000, 1.0000,
		] as [f32; 16];
		
		SlamCamera {
			view_matrix: glam::Mat4::from_rows_array(&iphone14_view_matrix),
			position: glam::Vec3::ZERO,
			rotation: glam::Quat::IDENTITY,
		}
	}
}

impl SlamCamera {
	
	
	fn get_transformation_matrix(&self) -> glam::Mat4 {
		glam::Mat4::from_rotation_translation(self.rotation, self.position)
	}
	
	
}
