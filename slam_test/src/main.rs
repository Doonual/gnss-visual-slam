use bevy::prelude::*;
use bevy::{input_focus::InputFocus,};
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, render::render_resource::TextureFormat};


use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use rand_core::Rng;

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
	app.add_plugins(EntropyPlugin::<WyRand>::default());
	app.add_plugins(ui::UIPlugin);
	app.add_plugins(slam::SlamPlugin);
	app.init_resource::<InputFocus>();
	//app.add_systems(Startup, spawn_scene.in_set(StartupSet::SpawnScene));
	app.add_systems(Startup, spawn_scene);
	app.add_systems(Startup, spawn_ui.after(spawn_scene));
	
	app.add_systems(PreUpdate, keyboard_control_system);
	app.add_systems(PreUpdate, quit_system);
	app.run();
	
}


pub fn spawn_ui(
	mut commands: Commands,
	slam_camera_query: Query<Entity, With<slam::SlamCamera>>,
) {
	
	commands.spawn(
		(
			Camera2d,
			RenderLayers::layer(2),
		)
	);
	//commands.spawn(button(&assets, image_handle));
	
	let Ok(slam_camera) = slam_camera_query.single() else {
		println!("Failed to find a SLAM Camera when spawning the UI. Make sure the spawn_ui system runs after the spawn_scene system");
		return;
	};
	
	// Spawn in ui
	commands.spawn(
		(
			Node {
				width: percent(100),
				height: percent(100),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			children![
				(
					Node {
						width: percent(50),
						aspect_ratio: Some(1.0),
						..default()
					},
					BackgroundColor(Color::srgb_u8(0, 255, 0)),
					ui::SlamCameraUI{target_camera: Some(slam_camera)},
				),
				(
					Node {
						width: percent(50),
						..default()
					},
					ui::MapCameraMonitor,
				),
			]
		)
	);
	
	
	
	
}



fn spawn_scene(
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
				//clear_color: Color::srgb_u8(230, 240, 255).into(),
				clear_color: Color::srgb_u8(0, 0, 0).into(),
				..default()
			},
			RenderTarget::Image(slam_camera_texture_handle.clone().into()),
			RenderLayers::layer(0),
			slam::SlamCamera,
			KeyboardControl{ move_speed: 5.0, rotation_speed: 0.05},
			Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
	commands.spawn(
		(
			Camera3d::default(),
			Camera {
				order: 0,
				clear_color: Color::srgb_u8(30, 30, 30).into(),
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
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(-0.5, -0.5, 0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(-0.5, 0.5, -0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(-0.5, 0.5, 0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(0.5, -0.5, -0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(0.5, -0.5, 0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(0.5, 0.5, -0.5),
					slam::WorldTrackPoint,
				),
				(
					Transform::from_xyz(0.5, 0.5, 0.5),
					slam::WorldTrackPoint,
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

fn quit_system(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut commands: Commands,
) {
	if keyboard_input.pressed(KeyCode::KeyQ) == true {
		commands.write_message(AppExit::Success);
	}
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
		
		if keyboard_input.pressed(KeyCode::KeyH) == true {
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyL) == true {
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * -keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyK) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * keyboard_control.rotation_speed) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyJ) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * -keyboard_control.rotation_speed) * transform.rotation;
		}
		
	}
	
	
}



