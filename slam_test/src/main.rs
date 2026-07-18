use bevy::prelude::*;
use bevy::{input_focus::InputFocus,};
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, render::render_resource::TextureFormat};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::bevy_egui::prelude::*;

mod ui;
mod slam;
// Todo: Rename the virtual stuff to map. Because they are in the map were building

#[derive(Component)]
pub struct KeyboardControl {
	move_speed: f32,
	rotation_speed: f32,
}


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSet {
	SpawnScene,
}


fn main() {
	
	let mut app = App::new();
	app.add_plugins(DefaultPlugins.set(WindowPlugin {
		primary_window: Some(Window {
			present_mode: bevy::window::PresentMode::AutoNoVsync,
			..default()
		}),
		..default()
	}));
	//app.add_plugins(EguiPlugin::default());
	//app.add_plugins(WorldInspectorPlugin::new());
	app.configure_sets(Startup, StartupSet::SpawnScene);
	//app.add_plugins(DefaultPlugins);
	//app.add_plugins(bevy_framepace::FramepacePlugin);
	app.add_plugins(ui::UIPlugin);
	app.init_resource::<InputFocus>();
	app.add_systems(Startup, setup.in_set(StartupSet::SpawnScene));
	app.add_systems(PreUpdate, keyboard_control_system);
	app.run();
	
}





fn setup(
	mut commands: Commands,
	//assets: Res<AssetServer>,
	mut images: ResMut<Assets<Image>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>
) {
	
	
	let slam_camera_texture = Image::new_target_texture(
		512,
		512,
		TextureFormat::Rgba8UnormSrgb,
		Some(TextureFormat::Rgba8UnormSrgb),
	);
	let slam_camera_texture_handle = images.add(slam_camera_texture);
	
	let virtual_camera_texture = Image::new_target_texture(
		512,
		512,
		TextureFormat::Rgba8UnormSrgb,
		Some(TextureFormat::Rgba8UnormSrgb),
	);
	let virtual_camera_texture_handle = images.add(virtual_camera_texture);
	
	commands.spawn(
		(
			Camera3d::default(),
			Camera {
				order: -1,
				clear_color: Color::WHITE.into(),
				..default()
			},
			RenderTarget::Image(slam_camera_texture_handle.clone().into()),
			RenderLayers::layer(0),
			slam::SLAMCamera,
			KeyboardControl{ move_speed: 10.0, rotation_speed: 0.05},
			Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
	commands.spawn(
		(
			Camera3d::default(),
			Camera {
				order: 0,
				clear_color: Color::srgb_u8(0, 0, 255).into(),
				..default()
			},
			RenderTarget::Image(virtual_camera_texture_handle.clone().into()),
			RenderLayers::layer(1),
			slam::MapCamera,
			//KeyboardControl{ move_speed: 10.0, rotation_speed: 0.05},
			Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
	
	
	commands.spawn(
		(
			Mesh3d(meshes.add(Circle::new(4.0))),
			MeshMaterial3d::<StandardMaterial>(materials.add(StandardMaterial {base_color: Color::WHITE, ..default()})),
			Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
			RenderLayers::layer(0).with(1),
		)
	);
	
	commands.spawn(
		(
			Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
			MeshMaterial3d::<StandardMaterial>(materials.add(StandardMaterial {base_color: Color::srgb_u8(124, 144, 255), ..default()})),
			Transform::from_xyz(0.0, 0.5, 0.0),
			RenderLayers::layer(0).with(1),
			children![
				(
					Transform::from_xyz(-0.5, -0.5, -0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(-0.5, -0.5, 0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(-0.5, 0.5, -0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(-0.5, 0.5, 0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(0.5, -0.5, -0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(0.5, -0.5, 0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(0.5, 0.5, -0.5),
					slam::WorldTrackPoint(0),
				),
				(
					Transform::from_xyz(0.5, 0.5, 0.5),
					slam::WorldTrackPoint(0),
				),
				
			]
		)
	);
	commands.spawn(
		(
			PointLight {
				shadow_maps_enabled: true,
				..default()
			},
			Transform::from_xyz(4.0, 8.0, 4.0),
			RenderLayers::layer(0).with(1), // Might not need render layer 0
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
		
		if keyboard_input.pressed(KeyCode::ArrowLeft) == true {
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::ArrowRight) == true {
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * -keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::ArrowUp) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::ArrowDown) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * -keyboard_control.rotation_speed) * transform.rotation;
		}
		
	}
	
	
}



