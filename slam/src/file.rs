use std::fmt;
use glam::{Vec4, Vec3, Vec2, Quat, Mat4};
use json::{self, JsonValue};

pub trait Saveable where Self : Sized {
	fn to_json(&self) -> JsonValue;
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError>;
	fn from_json_or_default(json_value: &JsonValue) -> Self;
}

#[derive(Debug)]
pub enum DecodeJsonError {
	ParameterNotFound {
		json_searched: JsonValue,
		parameter_name: String,
	},
	UnexpectedValue {
		json_searched: JsonValue,
		parameter_name: String,
		value: String,
	},
}
impl fmt::Display for DecodeJsonError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		
		match self {
			DecodeJsonError::ParameterNotFound {
				json_searched,
				parameter_name,
			} => {
				write!(f, "Parameter \"{parameter_name}\" not found in JSON \"{json_searched}\"")
			}
			DecodeJsonError::UnexpectedValue {
				json_searched,
				parameter_name,
				value,
			} => {
				write!(f, "Unexpected value \"{value}\" for parameter \"{parameter_name}\" in JSON \"{json_searched}\"")
			}
		}
		
	}
}

impl Saveable for glam::Vec2 {
	
	fn to_json(&self) -> JsonValue {
		
		let mut vec2_json = JsonValue::new_object();
		vec2_json["x"] = self.x.into();
		vec2_json["y"] = self.y.into();
		vec2_json
		
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(x) = json_value["x"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "x".into(), });};
		let Some(y) = json_value["y"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "y".into(), });};
		
		Ok(Vec2 {
			x,
			y,
		})
		
	}
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		
		let x = json_value["x"].as_f32().unwrap_or(0.0);
		let y = json_value["y"].as_f32().unwrap_or(0.0);
		
		Vec2 {
			x,
			y,
		}
		
	}
	
}

impl Saveable for glam::Vec3 {
	
	fn to_json(&self) -> JsonValue {
		
		let mut vec3_json = JsonValue::new_object();
		vec3_json["x"] = self.x.into();
		vec3_json["y"] = self.y.into();
		vec3_json["z"] = self.z.into();
		vec3_json
		
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(x) = json_value["x"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "x".into() });};
		let Some(y) = json_value["y"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "y".into() });};
		let Some(z) = json_value["z"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "z".into() });};
		
		Ok(Vec3 {
			x,
			y,
			z,
		})
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let x = json_value["x"].as_f32().unwrap_or(0.0);
		let y = json_value["y"].as_f32().unwrap_or(0.0);
		let z = json_value["z"].as_f32().unwrap_or(0.0);
		
		Vec3 {
			x,
			y,
			z,
		}
	}
	
}

impl Saveable for Vec4 {
	
	fn to_json(&self) -> JsonValue {
		
		let mut vec4_json = JsonValue::new_object();
		vec4_json["w"] = self.w.into();
		vec4_json["x"] = self.x.into();
		vec4_json["y"] = self.y.into();
		vec4_json["z"] = self.z.into();
		vec4_json
		
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(w) = json_value["w"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "w".into() });};
		let Some(x) = json_value["x"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "x".into() });};
		let Some(y) = json_value["y"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "y".into() });};
		let Some(z) = json_value["z"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "z".into() });};
		
		Ok(Vec4::new(x, y, z, w))
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let w = json_value["w"].as_f32().unwrap_or(0.0);
		let x = json_value["x"].as_f32().unwrap_or(0.0);
		let y = json_value["y"].as_f32().unwrap_or(0.0);
		let z = json_value["z"].as_f32().unwrap_or(0.0);
		
		Vec4::new(x, y, z, w)
	}
	
}


impl Saveable for glam::Quat {
	
	fn to_json(&self) -> JsonValue {
		
		let mut quat_json = JsonValue::new_object();
		quat_json["w"] = self.w.into();
		quat_json["x"] = self.x.into();
		quat_json["y"] = self.y.into();
		quat_json["z"] = self.z.into();
		quat_json
		
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let Some(w) = json_value["w"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "w".into() });};
		let Some(x) = json_value["x"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "x".into() });};
		let Some(y) = json_value["y"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "y".into() });};
		let Some(z) = json_value["z"].as_f32() else {return Err(DecodeJsonError::ParameterNotFound {json_searched: json_value.clone(), parameter_name: "z".into() });};
		
		Ok(Quat::from_xyzw(x, y, z, w))
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		let Some(w) = json_value["w"].as_f32() else {return Quat::IDENTITY;};
		let Some(x) = json_value["x"].as_f32() else {return Quat::IDENTITY;};
		let Some(y) = json_value["y"].as_f32() else {return Quat::IDENTITY;};
		let Some(z) = json_value["z"].as_f32() else {return Quat::IDENTITY;};
		
		Quat::from_xyzw(x, y, z, w)
	}
	
}


impl Saveable for glam::Mat4 {
	
	fn to_json(&self) -> JsonValue {
		
		let mut quat_json = JsonValue::new_object();
		quat_json["col_0"] = self.col(0).to_json();
		quat_json["col_1"] = self.col(1).to_json();
		quat_json["col_2"] = self.col(2).to_json();
		quat_json["col_3"] = self.col(3).to_json();
		quat_json
		
	}
	fn from_json(json_value: &JsonValue) -> Result<Self, DecodeJsonError> {
		
		let col_0 = glam::Vec4::from_json(&json_value["col_0"])?;
		let col_1 = glam::Vec4::from_json(&json_value["col_1"])?;
		let col_2 = glam::Vec4::from_json(&json_value["col_2"])?;
		let col_3 = glam::Vec4::from_json(&json_value["col_3"])?;
		
		Ok(Mat4::from_cols(col_0, col_1, col_2, col_3))
		
	}
	
	fn from_json_or_default(json_value: &JsonValue) -> Self {
		
		let Ok(col_0) = glam::Vec4::from_json(&json_value["col_0"]) else {return Mat4::IDENTITY; };
		let Ok(col_1) = glam::Vec4::from_json(&json_value["col_1"]) else {return Mat4::IDENTITY; };
		let Ok(col_2) = glam::Vec4::from_json(&json_value["col_2"]) else {return Mat4::IDENTITY; };
		let Ok(col_3) = glam::Vec4::from_json(&json_value["col_3"]) else {return Mat4::IDENTITY; };
		Mat4::from_cols(col_0, col_1, col_2, col_3)
	}
	
}

