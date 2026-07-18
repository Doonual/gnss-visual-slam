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
struct MapCameraPreview;

#[derive(Component)]
#[require(Node)]
struct TrackPointUI;

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
		app.add_systems(Update, link_camera_previews);
		app.add_systems(Update, populate_slam_preview_tree);
		//app.add_systems(Update, spawn_slam_preview_track_point_ui);
		//app.add_systems(Update, spawn_slam_preview_track_point_ui_crosshair);
		app.add_systems(Startup, spawn_ui.before(link_camera_previews));
		//app.add_systems(Startup, link_camera_previews.in_set(StartupSet::LinkUI));
		app.add_systems(Update, draw_track_point_gizmos);
		
		
	}
}



fn link_camera_previews(
	slam_camera_query: Query<&RenderTarget,With<slam::SLAMCamera>>,
	virtual_camera_query: Query<&RenderTarget, With<slam::MapCamera>>,
	slam_preview_query: Query<&mut ImageNode, (With<SLAMCameraMonitor>, Without<MapCameraPreview>)>,
	virtual_preview_query: Query<&mut ImageNode, (With<MapCameraPreview>, Without<SLAMCameraMonitor>)>,
	input: Res<ButtonInput<KeyCode>>,
) {
	
	if input.just_pressed(KeyCode::KeyT) {
		match slam_camera_query.single() {
			Ok(camera) => {
				match camera.as_image() {
					Some(image) => {
						for mut slam_preview in slam_preview_query{
							slam_preview.image = image.clone();
							println!("Linked SLAM Camera output to SLAM camera preview");
						}
					},
					None => {println!("Found no image handle in RenderTarget component in entity with SLAMCamera component")},
				}
			},
			Err(_) => {println!("Found no RenderTarget component on entity with SLAMCamera component");},
		}
		
		
		match virtual_camera_query.single() {
			Ok(camera) => {
				match camera.as_image() {
					Some(image) => {
						for mut virtual_preview in virtual_preview_query {
							virtual_preview.image = image.clone();
							println!("Linked virtual camera output to virtual camera preview");
						}
					},
					None => {},
				}
			},
			Err(_) => {},
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
						..default()
					},
					SLAMCameraUI,
				),
				(
					Node {
						width: percent(50),
						..default()
					},
					MapCameraPreview,
				),
			]
		)
	);
	
	
	
	
}

fn populate_slam_preview_tree(
	mut commands: Commands,
	query: Query<Entity, Added<SLAMCameraUI>>,
	
) {
	
	for entity in &query {
		commands.entity(entity).with_children(|parent| {
			parent.spawn((
				SLAMCameraMonitor,
				Node {
					width: percent(100),
					height: percent(100),
					..default()
				}
			));
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






