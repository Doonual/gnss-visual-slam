use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, prelude::*, render::render_resource::TextureFormat};
use bevy::input::keyboard::*;
use bevy::time;


#[derive(Component)]
struct KeyboardControl {
	move_speed: f32,
	rotation_speed: f32,
}


fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_systems(Startup, scene.spawn());
	app.add_systems(Startup, spawn_camera);
	app.add_systems(Update, keyboard_control_system);
	app.run();
	
	
}

fn spawn_camera(mut commands: Commands) {
	
	commands.spawn(
		(
			Camera3d::default(),
			KeyboardControl{ move_speed: 10.0, rotation_speed: 1.0},
			Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
}

fn keyboard_control_system(
	mut query: Query<(&mut Transform, &KeyboardControl)>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
	fixed_time: Res<Time<Fixed>>
) {
	
	for (mut transform, keyboard_control) in &mut query {
		
		let transform_forward = transform.forward();
		let transform_right = transform.right();
		let transform_up = transform.up();
		
		
		if keyboard_input.pressed(KeyCode::KeyW) == true {
			transform.translation += transform_forward * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		if keyboard_input.pressed(KeyCode::KeyS) == true {
			transform.translation -= transform_forward * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		if keyboard_input.pressed(KeyCode::KeyD) == true {
			transform.translation += transform_right * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		if keyboard_input.pressed(KeyCode::KeyA) == true {
			transform.translation -= transform_right * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		if keyboard_input.pressed(KeyCode::Space) == true {
			transform.translation += transform_up * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		if keyboard_input.pressed(KeyCode::ShiftLeft) == true {
			transform.translation -= transform_up * fixed_time.delta_secs() * keyboard_control.move_speed;
		}
		
	}
	
	
}




fn scene() -> impl SceneList {
	
	bsn_list! [
		(
			#CircularBase
			Mesh3d(asset_value(Circle::new(4.0)))
			MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
			Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
		),
		(
			#Cube
			Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
			MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
			Transform::from_xyz(0.0, 0.5, 0.0)
		),
		(
			PointLight {
				shadow_maps_enabled: true,
			}
			Transform::from_xyz(4.0, 8.0, 4.0)
		),
	]
	
}



