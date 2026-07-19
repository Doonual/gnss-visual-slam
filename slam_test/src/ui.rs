use bevy::prelude::*;
use bevy::{color::palettes::basic::*,input_focus::{FocusCause, InputFocus},};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;

use crate::slam;

#[derive(Component)]
#[require(Node)]
#[require(ImageNode)]
struct SLAMCameraMonitor;

#[derive(Component)]
#[require(Node)]
#[require(ImageNode)]
struct MapCameraMonitor;

#[derive(Component)]
#[require(Node)]
struct TrackPointUI;

fn move_track_point_ui(
	slam_camera_query: Query<(&Camera, &GlobalTransform), With<slam::SLAMCamera>>,
	mut track_point_query: Query<&mut Node, With<TrackPointUI>>,
	world_track_point_query: Query<&GlobalTransform, With<slam::WorldTrackPoint>>,
) {
	
	let Ok((slam_camera, slam_camera_transform)) = slam_camera_query.single() else {return;};
	
	
	for (mut track_point_crosshair_node, world_track_point_transform) in track_point_query.iter_mut().zip(world_track_point_query.iter()) {
		
		let Ok(viewport_pos) = slam_camera.world_to_viewport(slam_camera_transform, world_track_point_transform.translation()) else {continue; };
		
		let Some(viewport_size) = slam_camera.logical_viewport_size() else {continue;};
		
		let viewport_pos_percent = viewport_pos / viewport_size;
		
		track_point_crosshair_node.left = percent(viewport_pos_percent.x * 100.0);
		track_point_crosshair_node.top = percent(viewport_pos_percent.y * 100.0);
	}
	
	
	
}



#[derive(Component)]
struct SLAMCameraUI;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSet {
	LinkUI,
}


#[derive(Default, Reflect, GizmoConfigGroup)]
struct UIGizmos;

pub struct UIPlugin;
impl Plugin for UIPlugin {
	fn build(&self, app: &mut App) {
		
		app.configure_sets(Startup, StartupSet::LinkUI.after(crate::StartupSet::SpawnScene));
		
		app.insert_gizmo_config(
			UIGizmos,
			GizmoConfig {
				render_layers: RenderLayers::layer(2),
				..default()
			}
		);
		app.add_systems(Update, move_track_point_ui);
		app.add_systems(Update, link_slam_camera_monitor);
		app.add_systems(Update, link_map_camera_monitor);
		app.add_systems(Update, populate_slam_preview_tree);
		app.add_systems(Startup, spawn_ui);
		//app.add_systems(Update, spawn_slam_preview_track_point_ui);
		//app.add_systems(Update, spawn_slam_preview_track_point_ui_crosshair);
		//app.add_systems(Startup, link_camera_previews.in_set(StartupSet::LinkUI));
		app.add_systems(Update, draw_track_point_gizmos);
		
		
	}
}

fn link_slam_camera_monitor(
	slam_camera_query: Query<&RenderTarget, With<slam::SLAMCamera>>,
	slam_monitor_query: Query<&mut ImageNode, With<SLAMCameraMonitor>>,
) {
	let Ok(single_camera) = slam_camera_query.single() else {return;};
	let Some(slam_camera_image_handle) = single_camera.as_image() else {return;};
	
	
	for mut slam_monitor in slam_monitor_query {
		if slam_monitor.image != *slam_camera_image_handle {
			slam_monitor.image = slam_camera_image_handle.clone();
		}
	}
	
}

fn link_map_camera_monitor(
	map_camera_query: Query<&RenderTarget, With<slam::MapCamera>>,
	map_monitor_query: Query<&mut ImageNode, With<MapCameraMonitor>>,
) {
	let Ok(single_camera) = map_camera_query.single() else {return;};
	let Some(map_camera_image_handle) = single_camera.as_image() else {return;};
	
	
	for mut map_monitor in map_monitor_query {
		if map_monitor.image != *map_camera_image_handle {
			map_monitor.image = map_camera_image_handle.clone();
		}
	}
	
}




pub fn draw_track_point_gizmos(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<slam::WorldTrackPoint>>) {
	
	for transform in query {
		let mut sphere_isometry = Isometry3d::from_translation(transform.translation());
		sphere_isometry.rotation = transform.rotation();
		gizmos.sphere(sphere_isometry, 0.1, GREEN);
	}
}

pub fn spawn_ui(
	mut commands: Commands,
	//mut assets: Res<AssetServer>,
	//mut images: ResMut<Assets<Image>>,
	//mut meshes: ResMut<Assets<Mesh>>,
	//mut materials: ResMut<Assets<StandardMaterial>>
) {
	
	commands.spawn(
		(
			Camera2d,
			RenderLayers::layer(2),
		)
	);
	//commands.spawn(button(&assets, image_handle));
	
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
					SLAMCameraUI,
				),
				(
					Node {
						width: percent(50),
						..default()
					},
					MapCameraMonitor,
				),
			]
		)
	);
	
	
	
	
}

fn populate_slam_preview_tree(
	mut commands: Commands,
	query: Query<Entity, Added<SLAMCameraUI>>,
	
) {
	
	let crosshair_size = 20.0; // Px
	
	for entity in &query {
		commands.entity(entity).with_children(|parent| {
			parent.spawn((
				SLAMCameraMonitor,
				Node {
					width: percent(100.0),
					height: percent(100.0),
					..default()
				},
				ImageNode {
					image_mode: NodeImageMode::Stretch,
					..default()
				},
			));
			
			for i in 0..8 {
				parent.spawn((
					TrackPointUI,
					Node {
						width: Val::Px(crosshair_size),
						height: Val::Px(crosshair_size),
						
						position_type: PositionType::Absolute,
						left: Val::Percent(50.0),
						top: Val::Percent(50.0),
						..default()
					},
					UiTransform::from_translation(Val2::new(
					    px(-0.5 * crosshair_size),
					    px(-0.5 * crosshair_size),
					))
				)).with_children(|parent| {
					// Horizontal line
					parent.spawn((
						Node {
							width: Val::Px(20.0),
							height: Val::Px(2.0),
							position_type: PositionType::Absolute,
							top: Val::Px(9.0),
							..default()
						},
						BackgroundColor(Color::srgb_u8(255, 0, 0)),
					));
					
					// Vertical line
					parent.spawn((
						Node {
							width: Val::Px(2.0),
							height: Val::Px(20.0),
							position_type: PositionType::Absolute,
							left: Val::Px(9.0),
							..default()
						},
						BackgroundColor(Color::srgb_u8(255, 0, 0)),
					));
				});
			}
			println!("Spawned track point and crosshair children");
		});
	}
	
}

// Old
/*
fn spawn_slam_preview_track_point_ui(
	mut commands: Commands,
	query: Query<Entity, With<SLAMCameraPreview>>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
) {
	
	if keyboard_input.just_pressed(KeyCode::KeyT) == true {
		
		
		let max_trackers = 8;
		
		for entity in &query {
			commands.entity(entity).with_children(|parent| {
				for _ in 0..max_trackers {
					
					parent.spawn((
						TrackPointUI,
						Node {
							width: Val::Px(20.0),
							height: Val::Px(20.0),
							position_type: PositionType::Absolute,
							left: Val::Percent(50.0),
							top: Val::Percent(50.0),
							..default()
						},
					)).with_children(|parent| {
						// Horizontal line
						parent.spawn((
							Node {
								width: Val::Px(20.0),
								height: Val::Px(2.0),
								position_type: PositionType::Absolute,
								top: Val::Px(9.0),
								..default()
							},
							BackgroundColor(Color::srgb_u8(255, 0, 0)),
						));
						
						// Vertical line
						parent.spawn((
							Node {
								width: Val::Px(2.0),
								height: Val::Px(20.0),
								position_type: PositionType::Absolute,
								left: Val::Px(9.0),
								..default()
							},
							BackgroundColor(Color::srgb_u8(255, 0, 0)),
						));
					});
					println!("Spawned track point and crosshair children");
					
				}
			});
		}
		
	}
	
}
*/






