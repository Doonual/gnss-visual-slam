use bevy::prelude::*;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;

use dear_imgui_bevy::prelude::*;
use dear_imgui_rs::Ui;

use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalRng;
use rand_core::Rng;
use bevy::math::ops::*;

use crate::slam;


#[derive(Default, Reflect, GizmoConfigGroup)]
struct MapCameraGizmoConfigGroup;


// This component monitors the SLAM Camera
#[derive(Component, Default)]
#[require(Node)]
#[require(ImageNode)]
struct SlamCameraMonitor;

// Links the SLAM camera output to all the SLAM camera monitors
fn link_slam_camera_monitor(
	slam_camera_query: Query<&RenderTarget, With<slam::SlamCamera>>,
	slam_camera_ui_query: Query<&SlamCameraUI>,
	slam_monitor_query: Query<(&ChildOf, &mut ImageNode), With<SlamCameraMonitor>>,
) {
	
	/*
	let Ok(single_camera) = slam_camera_query.single() else {return;};
	let Some(slam_camera_image_handle) = single_camera.as_image() else {return;};
	
	hkk
	for mut slam_monitor in slam_monitor_query {
		if slam_monitor.image != *slam_camera_image_handle {
			slam_monitor.image = slam_camera_image_handle.clone();
		}
	}
	*/
	
	for (slam_monitor_parent, mut slam_monitor_imagenode) in slam_monitor_query {
		
		let Ok(slam_camera_ui) = slam_camera_ui_query.get(slam_monitor_parent.parent()) else {
			println!("Failed to link SLAM Camera to SLAM camera monitor: SlamCameraMonitor is not child of SlamCameraUI");
			continue;
		};
		let Some(slam_camera_monitor_target_camera) = slam_camera_ui.target_camera else {
			println!("Failed to link SLAM camera to SLAM camera monitor: Target camera not specified");
			continue;
		};
		let Ok(slam_camera_render_target) = slam_camera_query.get(slam_camera_monitor_target_camera) else {
			println!("Failed to link SLAM camera to SLAM camera monitor: Target camera not found / missing SlamCamera component");
			continue;
		};
		let Some(slam_camera_image_handle) = slam_camera_render_target.as_image() else {
			println!("Failed to link SLAM camera to SLAM camera monitor: Target camera has no image handle");
			continue;
		};
		
		if slam_monitor_imagenode.image != *slam_camera_image_handle {
			println!("Linked SLAM camera to SLAM camera monitor");
			slam_monitor_imagenode.image = slam_camera_image_handle.clone();
		}
		
		
	}
	
}






// This component monitors the map camera
#[derive(Component)]
#[require(Node)]
#[require(ImageNode)]
pub struct MapCameraMonitor;

// Links the Map camera output to all the map camera monitors
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



// System that draws world track points in the slam camera
pub fn draw_track_point_gizmos(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<slam::WorldTrackPoint>>) {
	
	for transform in query {
		let mut sphere_isometry = Isometry3d::from_translation(transform.translation());
		sphere_isometry.rotation = transform.rotation();
		gizmos.sphere(sphere_isometry, 0.1, Color::srgb_u8(0,255,0));
	}
}

// System that draws slam keyframes in the map camera
pub fn draw_slam_keyframe_gizmos(mut gizmos: Gizmos<MapCameraGizmoConfigGroup>, query: Query<(&GlobalTransform, &slam::SlamKeyframe)>) {
	
	let gizmo_width = 0.5;
	let gizmo_height = 0.5;
	let gizmo_depth = 0.25;
	
	for (transform, slam_keyframe) in query {
		
		let position = transform.transform_point(slam_keyframe.position_uncertainty.mean);
		let tracker_isometry = Isometry3d {translation: position.to_vec3a(), rotation: transform.rotation()};
		
		let col = Color::srgb_u8(0, 0, 255);
		
		slam::get_eigenvectors(&slam_keyframe.position_uncertainty.covariance);
		
		gizmos.line(tracker_isometry.transform_point(Vec3{ x: -0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3{ x: -0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3{ x: -0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3{ x: 0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3{ x: 0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3{ x: 0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3{ x: 0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3{ x: -0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), col);
		
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: -0.5 * gizmo_width, y: 0.5 * gizmo_height * 1.2, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.5 * gizmo_width, y: 0.5 * gizmo_height * 1.2, z: -gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.5 * gizmo_height * 1.8, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.5 * gizmo_width, y: 0.5 * gizmo_height * 1.2, z: -gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: -0.5 * gizmo_width, y: 0.5 * gizmo_height * 1.2, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.5 * gizmo_height * 1.8, z: -gizmo_depth}).to_vec3(), col);
		
		
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: -0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.0, z: 0.0 * gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: -0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.0, z: 0.0 * gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: 0.5 * gizmo_width, y: -0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.0, z: 0.0 * gizmo_depth}).to_vec3(), col);
		gizmos.line(tracker_isometry.transform_point(Vec3 {x: 0.5 * gizmo_width, y: 0.5 * gizmo_height, z: -gizmo_depth}).to_vec3(), tracker_isometry.transform_point(Vec3 {x: 0.0, y: 0.0, z: 0.0 * gizmo_depth}).to_vec3(), col);
		
		
		
	}
	
}

pub fn draw_slam_keyframe_projected_tracker_gizmos(mut gizmos: Gizmos<MapCameraGizmoConfigGroup>, query: Query<(&slam::ProjectedTracker)>, slam_keyframe_query: Query<(&GlobalTransform, &slam::SlamKeyframe)>) {
	
	for projected_tracker in query {
		
		let Ok((slam_keyframe_transform, slam_keyframe)) = slam_keyframe_query.get(projected_tracker.keyframe) else {
			println!("Failed to draw projected tracker, owned keyframe not found");
			continue;
		};
		
		let camera_matrix = slam_keyframe.camera_projection;
		let project_point = Vec2{x: projected_tracker.tracked2d_pos.x - 0.5, y: projected_tracker.tracked2d_pos.y - 0.5};
		let projected_point = camera_matrix.inverse().project_point3(Vec3{x: 2.0 * project_point.x, y: -2.0 * project_point.y, z: 1.0});
		
		let mut eigenvectors = slam::get_eigenvectors(&projected_tracker.local_pose.covariance);
		
		let Some(mut eigenvector_a) = eigenvectors.pop() else {
			println!("Failed to draw projected tracker, missing eigenvector");
			continue;
		};
		let Some(mut eigenvector_b) = eigenvectors.pop() else {
			println!("Failed to draw projected tracker, missing eigenvector");
			continue;
		};
		let Some(mut eigenvector_c) = eigenvectors.pop() else {
			println!("Failed to draw projected tracker, missing eigenvector");
			continue;
		};
		
		eigenvector_a = slam_keyframe_transform.transform_point(eigenvector_a) - slam_keyframe_transform.translation();
		eigenvector_b = slam_keyframe_transform.transform_point(eigenvector_b) - slam_keyframe_transform.translation();
		eigenvector_c = slam_keyframe_transform.transform_point(eigenvector_c) - slam_keyframe_transform.translation();
		
		let quat_a = Quat::from_mat3(&Mat3::from_cols(eigenvector_b.normalize(), eigenvector_c.normalize(), eigenvector_a.normalize()));
		let quat_b = Quat::from_mat3(&Mat3::from_cols(eigenvector_c.normalize(), eigenvector_a.normalize(), eigenvector_b.normalize()));
		let quat_c = Quat::from_mat3(&Mat3::from_cols(eigenvector_a.normalize(), eigenvector_b.normalize(), eigenvector_c.normalize()));
		let isometry_c = Isometry3d{translation: slam_keyframe_transform.transform_point(projected_tracker.local_pose.mean).to_vec3a(), rotation: quat_c };
		let isometry_a = Isometry3d{translation: slam_keyframe_transform.transform_point(projected_tracker.local_pose.mean).to_vec3a(), rotation: quat_a };
		let isometry_b = Isometry3d{translation: slam_keyframe_transform.transform_point(projected_tracker.local_pose.mean).to_vec3a(), rotation: quat_b };
		
		let num = 5;
		for i in -num..num+1 {
			
			let ammount = i as f32 / (num + 1) as f32;
			let scale = sqrt(1.0 - ammount * ammount);
			
			let new_isometry_a = Isometry3d{translation: isometry_a.translation + ammount * eigenvector_a.to_vec3a(), rotation: isometry_a.rotation};
			gizmos.ellipse(new_isometry_a, Vec2{x: eigenvector_c.length() * scale, y: eigenvector_b.length() * scale}, Color::srgb_u8(200, 200, 200));
			
			let new_isometry_b = Isometry3d{translation: isometry_b.translation + ammount * eigenvector_b.to_vec3a(), rotation: isometry_b.rotation};
			gizmos.ellipse(new_isometry_b,Vec2{x: eigenvector_c.length() * scale, y: eigenvector_a.length() * scale}, Color::srgb_u8(200, 200, 200));
			
			let new_isometry_c = Isometry3d{translation: isometry_c.translation + ammount * eigenvector_c.to_vec3a(), rotation: isometry_c.rotation};
			gizmos.ellipse(new_isometry_c, Vec2{x: eigenvector_a.length() * scale, y: eigenvector_b.length() * scale}, Color::srgb_u8(200, 200, 200));
		}
		
		//println!("Eigenvector A: {eigenvector_a}");
		//println!("Eigenvector B: {eigenvector_b}");
		//println!("Eigenvector C: {eigenvector_c}");
		
		//gizmos.line(slam_keyframe_transform.translation(), slam_keyframe_transform.transform_point(projected_point * 100.0), Color::srgb_u8(255, 0, 255));
		gizmos.line(isometry_c.translation.to_vec3(), isometry_c.translation.to_vec3() + eigenvector_a, Color::srgb_u8(255, 0, 0));
		gizmos.line(isometry_a.translation.to_vec3(), isometry_a.translation.to_vec3() + eigenvector_b, Color::srgb_u8(0, 255, 0));
		gizmos.line(isometry_b.translation.to_vec3(), isometry_b.translation.to_vec3() + eigenvector_c, Color::srgb_u8(0, 0, 255));
		
		
	}
	
}

// This component displays the screenspace trackers on the SLAM Camera
#[derive(Component)]
#[require(Node)]
struct TrackPointUI;

// Moves the Node componet attached to all the TrackPointUI components to their screenspace
// position.
fn move_track_point_ui(
	slam_camera_query: Query<(&Camera, &GlobalTransform), With<slam::SlamCamera>>,
	mut track_point_query: Query<&mut Node, With<TrackPointUI>>,
	track2d_query: Query<&slam::Track2D>,
	world_track_point_query: Query<&GlobalTransform, With<slam::WorldTrackPoint>>,
) {
	
	for (mut track_point_crosshair_node, track2d) in track_point_query.iter_mut().zip(track2d_query) {
		let viewport_pos_percent = track2d.viewport_position;
		
		track_point_crosshair_node.left = percent(viewport_pos_percent.x * 100.0);
		track_point_crosshair_node.top = percent(viewport_pos_percent.y * 100.0);
	}
	
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
pub struct SlamCameraUI {
	pub target_camera: Option<Entity>,
}

fn populate_slam_preview_tree(
	mut commands: Commands,
	query: Query<Entity, Added<SlamCameraUI>>,
) {
	
	
	
	let crosshair_size = 20.0; // Px
	
	for entity in &query {
		commands.entity(entity).with_children(|parent| {
			parent.spawn((
				SlamCameraMonitor{ ..default()},
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
			
			for _ in 0..8 {
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



pub struct UIPlugin;
impl Plugin for UIPlugin {
	fn build(&self, app: &mut App) {
		
		// The UI plugin shouldn't spawn in the ui components, only make them work
		// This will make it easier to specify how the UI links to the Scene
		
		app.add_systems(Update, link_slam_camera_monitor);
		app.add_systems(Update, link_map_camera_monitor);
		//app.add_systems(Update, draw_gaussian_gizmo);
		
		app.add_systems(Update, move_track_point_ui);
		app.add_systems(Update, populate_slam_preview_tree);
		//app.add_systems(Update, draw_track_point_gizmos);
		
		app.init_gizmo_group::<MapCameraGizmoConfigGroup>();
		app.add_systems(Update, draw_slam_keyframe_gizmos);
		app.add_systems(Update, draw_slam_keyframe_projected_tracker_gizmos);
		
		app.add_systems(Startup, setup_gizmo_config);
		
		app.insert_resource(ImguiState::default());
		if let Ok(primary_pass) = app.imgui_primary_pass() {
			if let Err(error) = app.add_imgui_systems(&primary_pass, primary_pass.system(imgui_draw_object_list)) {
				println!("Failed to add imgui system");
			}
			if let Err(error) = app.add_imgui_systems(&primary_pass, primary_pass.system(imgui_draw_inspector)) {
				println!("Failed to add imgui system");
			}
		}
		else {
			println!("Failed to get imgui primary pass");
		}
		
		
		
	}
}

fn setup_gizmo_config(mut config_store: ResMut<GizmoConfigStore>, mut commands: Commands) {
	
    // Put this gizmo group on layer 1
    let (config, _) = config_store.config_mut::<MapCameraGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(1);
	
}

fn draw_gaussian_gizmo(mut gizmos: Gizmos, mut rng: Single<&mut WyRand, With<GlobalRng>>) {
	
	let num_samples = 10000;
	
	for _ in 0..num_samples {
		
		let threshold = (rng.next_u32() % 1000000) as f32 / 1000000.0;
		let mut num_tries = 0;
		let mut found_valid_spot = false;
		while found_valid_spot == false && num_tries < 1 {
			
			
			num_tries += 1;
			let rand_x = ((rng.next_u32() % 1000000) as f32 / 1000000.0 - 0.5) * 10.0;
			let rand_y = ((rng.next_u32() % 1000000) as f32 / 1000000.0 - 0.5) * 10.0;
			let rand_z = ((rng.next_u32() % 1000000) as f32 / 1000000.0 - 0.5) * 10.0;
			
			
			let sample_pos = Vec3{x: rand_x, y: rand_y, z: rand_z};
			let gaussian_mean = Vec3{x: 0.0, y: 0.0, z: 0.0};
			let gaussian_covariance = Mat3::IDENTITY;
			let gaussian_sample = slam::sample_gaussian(&sample_pos, &gaussian_mean, &gaussian_covariance);
			
			if gaussian_sample < threshold {
				//println!("Retrying point, {gaussian_sample} < {threshold}");
				continue;
			}
			//println!("{gaussian_sample} > {threshold}");
			found_valid_spot = true;
			
			let isometry = Isometry3d {translation: Vec3{x: rand_x, y: rand_y, z: rand_z}.into(), rotation: Quat::IDENTITY};
			gizmos.cross(isometry, 0.03, Color::srgb_u8(0, 255, 0));
			
		}
		
	}
	
	
	//let point_isometry = Isometry3d{rotation: }
	//gizmos.cross()
	
}

#[derive(Resource)]
struct ImguiState {
	show_demo: bool,
	entity_in_inspector: Option<Entity>,
}
impl Default for ImguiState {
	fn default() -> ImguiState {
		return ImguiState {
			show_demo: false,
			entity_in_inspector: None,
		}
	}
}


fn imgui_draw_object_list (
	frame: ImguiFrame<'_>,
	mut imgui_state: ResMut<ImguiState>,
	slam_camera_query: Query<Entity, With<slam::SlamCamera>>,
	track2d_query: Query<(Entity, &slam::Track2D)>,
	slam_keyframe_query: Query<Entity, With<slam::SlamKeyframe>>,
	projected_tracker_query: Query<Entity, With<slam::ProjectedTracker>>,
) {
	
	let ui = frame.ui();
	
	let window = ui.window("SLAM Objects");
	
	
	window.build(|| {
		
		ui.checkbox("Show Demo", &mut imgui_state.show_demo);
		if imgui_state.show_demo {
			ui.show_demo_window(&mut imgui_state.show_demo);
		}
		
		ui.separator_with_text(format!("SLAM Cameras ({0})", slam_camera_query.count()));
		
		for (slam_camera_entity) in slam_camera_query {
			
			if let Some(test_test) = ui.tree_node(format!("SLAM Camera: {slam_camera_entity}")) {
				
				ui.separator_with_text(format!("Track2D List ({0})", track2d_query.count()));
				for (track2d_entity, track2d_component) in track2d_query {
					if track2d_component.owned_camera != Some(slam_camera_entity) {continue;}
					
					if ui.selectable_config(format!("Track 2D: {track2d_entity}")).selected(imgui_state.entity_in_inspector == Some(track2d_entity)).build() {
						imgui_state.entity_in_inspector = Some(track2d_entity);
					}
				}
			}
			
		}
		
		
		ui.separator_with_text(format!("SLAM Keyframe List ({0})", slam_keyframe_query.count()));
		for (slam_keyframe_entity) in slam_keyframe_query {
			if ui.selectable_config(format!("SLAM Keyframe {slam_keyframe_entity}")).selected(imgui_state.entity_in_inspector == Some(slam_keyframe_entity)).build() {
				imgui_state.entity_in_inspector = Some(slam_keyframe_entity);
			}
		}
		
		ui.separator_with_text(format!("Projected Tracker List ({0})", projected_tracker_query.count()));
		for (projected_tracker_entity) in projected_tracker_query {
			if ui.selectable_config(format!("Projected Tracker: {projected_tracker_entity}")).selected(imgui_state.entity_in_inspector == Some(projected_tracker_entity)).build() {
				imgui_state.entity_in_inspector = Some(projected_tracker_entity);
			}
		}
		
		
		
	});
	
}

pub trait imgui_inspector {
	fn draw(&self, ui: &Ui);
}

impl imgui_inspector for slam::SlamCamera {
	fn draw(&self, ui: &Ui) {
		
		ui.text("Hi");
		
	}
}

impl imgui_inspector for slam::Track2D {
	
	fn draw(&self, ui: &Ui) {
		
		ui.text(format!("Tracked position: {0}x{1}", self.viewport_position.x, self.viewport_position.y));
		
		
	}
	
}

impl imgui_inspector for slam::SlamKeyframe {
	fn draw(&self, ui: &Ui) {
		ui.text(format!("Position: {0}", self.position_uncertainty));
	}
}

impl imgui_inspector for slam::ProjectedTracker {
	fn draw(&self, ui: &Ui) {
		ui.text(format!("Local pose: {0}", self.local_pose));
	}
}

fn imgui_draw_inspector(
	frame: ImguiFrame<'_>,
	mut imgui_state: ResMut<ImguiState>,
	track2d_query: Query<(Entity, &slam::Track2D)>,
	slam_keyframe_query: Query<(Entity, &slam::SlamKeyframe)>,
	projected_tracker_query: Query<(Entity, &slam::ProjectedTracker)>,
) {
	
	let ui = frame.ui();
	
	ui.window("Inspector").build(|| {
		
		if let Some(found_entity_to_inspect) = imgui_state.entity_in_inspector {
			ui.text(format!("Selected entity: {0}", found_entity_to_inspect));
			
			if let Ok((_, component_to_inspect)) = track2d_query.get(found_entity_to_inspect) {
				component_to_inspect.draw(ui);
			}
			if let Ok((_, component_to_inspect)) = slam_keyframe_query.get(found_entity_to_inspect) {
				component_to_inspect.draw(ui);
			}
			if let Ok((_, component_to_inspect)) = projected_tracker_query.get(found_entity_to_inspect) {
				component_to_inspect.draw(ui);
			}
			
			
		}
		else {
			ui.text("Nothing selected");
		}
		
	});
	
	
}
