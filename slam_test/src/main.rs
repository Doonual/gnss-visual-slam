use bevy::prelude::*;
use bevy::{input_focus::InputFocus,};
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, render::render_resource::TextureFormat};
use bevy::window::WindowResolution;

use dear_imgui_bevy::prelude::*;
use dear_imgui_rs::{fonts::FontId, FontSource};

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
	enabled: i32,
}


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSet {
	SpawnScene,
}


fn main() -> Result {
	
	let mut window_resolution = WindowResolution::default();
	window_resolution.set_scale_factor_override(Some(1.0));
	
	let mut app = App::new();
	app.add_plugins(DefaultPlugins.set(WindowPlugin {
		primary_window: Some(Window {
			resolution: window_resolution,
			present_mode: bevy::window::PresentMode::AutoNoVsync,
			desired_maximum_frame_latency: core::num::NonZeroU32::new(1),
			..default()
		}),
		..default()
	}));
	
	app.try_install_imgui(ImguiPlugin::default());
	
	app.configure_sets(Startup, StartupSet::SpawnScene);
	app.add_plugins(EntropyPlugin::<WyRand>::default());
	app.add_plugins(ui::UIPlugin);
	app.add_plugins(slam::SlamPlugin);
	app.init_resource::<InputFocus>();
	app.add_systems(Startup, spawn_scene);
	app.add_systems(Startup, spawn_ui.after(spawn_scene));
	
	app.add_systems(PreUpdate, keyboard_control_system);
	app.add_systems(PreUpdate, quit_system);
	
	app.add_systems(Startup, setup_fonts);
	
	
	//app.add_systems(Update, imgui_example_ui);
	app.run();
	
	Ok(())
	
}


fn setup_fonts(mut contexts: NonSendMut<ImguiContexts>) {
	
	static FONT: &[u8] = include_bytes!("../assets/fonts/CommitMonoNerdFont-Bold.ttf");
	
	let id = contexts
	.primary_id()
	.expect("registry active")
	.expect("plugin installs a primary context");
	
	
	contexts
	.configure(id, |ctx| {
		unsafe {
			ctx.font_atlas().add_font(&[
				FontSource::ttf_data_with_size(FONT, 16.0),
			]);
		}
	})
	.expect("context should be configurable");
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
			Projection::Perspective(PerspectiveProjection {
				fov: 90.0 / 180.0 * 3.1415,
				..default()
			}),
			Camera {
				order: -1,
				//clear_color: Color::srgb_u8(230, 240, 255).into(),
				clear_color: Color::srgb_u8(0, 0, 0).into(),
				..default()
			},
			RenderTarget::Image(slam_camera_texture_handle.clone().into()),
			RenderLayers::layer(0),
			slam::SlamCamera,
			KeyboardControl{ move_speed: 5.0, rotation_speed: 3.0, enabled: 1},
			Transform::from_xyz(0.0, 1.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
	commands.spawn(
		(
			Camera3d::default(),
			Projection::Perspective(PerspectiveProjection {
				fov: 90.0 / 180.0 * 3.1415,
				..default()
			}),
			Camera {
				order: 0,
				clear_color: Color::srgb_u8(30, 30, 30).into(),
				..default()
			},
			RenderTarget::Image(virtual_camera_texture_handle.clone().into()),
			RenderLayers::layer(1),
			slam::MapCamera,
			KeyboardControl{ move_speed: 5.0, rotation_speed: 3.0, enabled: 0},
			Transform::from_xyz(0.0, 1.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y)
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
	mut query: Query<(&mut Transform, &mut KeyboardControl)>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
	fixed_time: Res<Time>
) {
	
	
	for (mut transform, mut keyboard_control) in &mut query {
		
		let transform_forward = transform.forward();
		let transform_right = transform.right();
		let transform_up = transform.up();
		
		if keyboard_input.just_pressed(KeyCode::Enter) == true {
			let new_enabled = 1 - keyboard_control.enabled;
			keyboard_control.enabled = new_enabled;
		}
		
		if keyboard_control.enabled == 0 {
			continue;
		}
		
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
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * keyboard_control.rotation_speed * fixed_time.delta_secs()) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyL) == true {
			transform.rotation = Quat::from_scaled_axis(Vec3::Y * -keyboard_control.rotation_speed * fixed_time.delta_secs()) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyK) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * keyboard_control.rotation_speed * fixed_time.delta_secs()) * transform.rotation;
		}
		if keyboard_input.pressed(KeyCode::KeyJ) == true {
			transform.rotation = Quat::from_scaled_axis(transform_right * -keyboard_control.rotation_speed * fixed_time.delta_secs()) * transform.rotation;
		}
		
		
	}
	
	
}


//fn imgui_example_ui(mut context: NonSendMut<ImguiContext>) {
	//let ui = context.ui();
	//let window = ui.window("hello world");
	//
	//window.build(|| {
		//ui.text("hello world!");
		//ui.text("this...is...bevy_mod_imgui!");
		//ui.separator();
		//let mouse_pos = ui.io().mouse_pos;
		//ui.text(format!(
			//"mouse position: ({:.1},{:.1})",
			//mouse_pos[0], mouse_pos[1]
		//));
	//});
	//
//}
