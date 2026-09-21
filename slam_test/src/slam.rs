use bevy::prelude::*;
use bevy::math::ops::*;

pub struct Gaussian3 {
	pub mean: Vec3,
	pub covariance: Mat3,
}



pub fn sample_gaussian_struct(sample_position: &Vec3, gaussian: Gaussian3) -> f32 {
	
	let covariance_inverted = gaussian.covariance.inverse();
	let relative_position = sample_position - gaussian.mean;
	let first_mat_mul = covariance_inverted.transpose() * relative_position;
	let inside_exp = -0.5 * (first_mat_mul.x * relative_position.x + first_mat_mul.y * relative_position.y + first_mat_mul.z * relative_position.z);
	
	return powf(2.0 * 3.141592, -3.0 / 2.0) * powf(covariance_inverted.determinant(), -0.5) * exp(inside_exp);
	
}
pub fn sample_gaussian(sample_position: &Vec3, mean: &Vec3, covariance: &Mat3) -> f32 {
	
	let covariance_inverted = covariance.inverse();
	let relative_position = sample_position - mean;
	let first_mat_mul = covariance_inverted.transpose() * relative_position;
	let inside_exp = -0.5 * (first_mat_mul.x * relative_position.x + first_mat_mul.y * relative_position.y + first_mat_mul.z * relative_position.z);
	
	//return 1.0 * exp(-sample_position.x * sample_position.x);
	//return 0.01 * exp(inside_exp);
	return powf(2.0 * 3.141592, -3.0 / 2.0) * powf(covariance_inverted.determinant(), -0.5) * exp(inside_exp);
	
}

// Component that designates an entity is a camera for collecting tracking points and making
// keyframes
#[derive(Component)]
#[require(GlobalTransform)]
#[require(Camera3d)]
pub struct SlamCamera;

// Active tracks on a camera's render plane


#[derive(Component)]
#[require(PointIndex)]
pub struct Track2D {
	pub viewport_position: Vec2,
	pub owned_camera: Option<Entity>,
	track_source: Option<Entity>,
}



#[derive(Component, Default)]
#[require(PointIndex)]
pub struct Track2DSource {
	track_point_destinations: Vec<Entity>,
}


// A tag that identifies each point
// We might want to track a point from multiple angles with multiple trackers
// Having this allows us to identify which trackers track the same point.
#[derive(Component, Default)]
pub struct PointIndex(pub u32);

#[derive(Resource)]
struct PointIndexCounter(u32);

fn assign_tracker_index(
	query: Query<&mut PointIndex, Added<PointIndex>>,
	mut point_index_counter: ResMut<PointIndexCounter>,
) {
	
	for mut pointindex in query {
		if pointindex.0 == 0 {
			point_index_counter.0 += 1;
			pointindex.0 = point_index_counter.0;
			let print_var = pointindex.0;
			println!("Assigned tracker index {print_var} to new point");
		}
	}
	
}

#[derive(Component)]
pub struct MapCamera;

#[derive(Component)]
#[require(Transform)]
#[require(Track2DSource)]
pub struct WorldTrackPoint;


fn apply_worldtrackpoint_tracks(
	mut commands: Commands,
	mut track2dsource_query: Query<(Entity, &mut Track2DSource, &GlobalTransform, &PointIndex), With<WorldTrackPoint>>,
	slam_camera_query: Query<(Entity, &Camera, &GlobalTransform), With<SlamCamera>>,
	mut track2d_query: Query<(Entity, &mut Track2D)>,
) {
	
	// Spawning new Track2D components
	for (track2dsource_entity, mut track2dsource, worldtrackpoint_globaltransform, worldtrackpoint_pointindex) in &mut track2dsource_query {
		for (slam_camera_entity, camera, camera_globaltransform) in &slam_camera_query{
			
			
			let Ok(projected_position) = camera.world_to_viewport(camera_globaltransform, worldtrackpoint_globaltransform.translation()) else {continue;};
			let Some(viewport_size) = camera.logical_viewport_size() else {continue;};
			let viewport_position = projected_position / viewport_size;
			
			if viewport_position.x < 0.0 || viewport_position.x > 1.0 || viewport_position.y < 0.0 || viewport_position.y > 1.0 {continue;}
			
			let mut tracker_exists = false;
			for (_, mut track2d) in &mut track2d_query {
				if track2d.owned_camera == Some(slam_camera_entity) && track2d.track_source == Some(track2dsource_entity) {
					track2d.viewport_position = viewport_position.clone();
					tracker_exists = true;
					break;
				}
			}
			
			if tracker_exists == false {
				
				let track2d_entity = commands.spawn(
					(
						Track2D {
							viewport_position: viewport_position,
							owned_camera: Some(slam_camera_entity),
							track_source: Some(track2dsource_entity),
						},
						PointIndex(worldtrackpoint_pointindex.0),
					)
				).id();
				track2dsource.track_point_destinations.push(track2d_entity);
				//println!("Spawned new Track2D from WorldTrackPoint");
				
			}
			
		}
	}
	
	// Check for invalid Track2D
	for (track2d_entity, mut track2d) in track2d_query {
		
		let Some(track_source_entity) = track2d.track_source.clone() else {
			commands.entity(track2d_entity).despawn();
			//println!("Despawned Track2D from WorldTrackPoint. Track source missing.");
			continue;
		};
		let Ok((_, mut track2dsource, worldtrackpoint_globaltransform, _)) = track2dsource_query.get_mut(track_source_entity) else {
			commands.entity(track2d_entity).despawn();
			//println!("Despawned Track2D from WorldTrackPoint. Invalid track source.");
			continue;
		};
		
		let Some(owned_camera_entity) = track2d.owned_camera.clone() else {
			track2dsource.track_point_destinations.retain(|&e| e != track2d_entity);
			commands.entity(track2d_entity).despawn();
			//println!("Despawned Track2D from WorldTrackPoint. Owner camera missing.");
			continue;
		};
		
		
		let Ok((_, camera, camera_globaltransform)) = slam_camera_query.get(owned_camera_entity) else {
			track2dsource.track_point_destinations.retain(|&e| e != track2d_entity);
			commands.entity(track2d_entity).despawn();
			//println!("Despawned Track2D. Invalid owner camera.");
			continue;
		};
		
		
		let Ok(projected_position) = camera.world_to_viewport(camera_globaltransform, worldtrackpoint_globaltransform.translation()) else {continue;};
		let Some(viewport_size) = camera.logical_viewport_size() else {continue;};
		let viewport_position = projected_position / viewport_size;
		
		if viewport_position.x < 0.0 || viewport_position.x > 1.0 || viewport_position.y < 0.0 || viewport_position.y > 1.0 {
			track2dsource.track_point_destinations.retain(|&e| e != track2d_entity);
			commands.entity(track2d_entity).despawn();
			//println!("Despawned Track2D from WorldTrackPoint. Outside owner camera frustum.");
			continue;
		}
		
	}
	
}


// The entity this is attached to will be a recorded camera pose
// All of it's children will be recordings of track points
#[derive(Component)]
#[require(GlobalTransform)]
pub struct SlamKeyframe {
	pub position_uncertainty: Gaussian3,
	pub rotation_uncertainty: Gaussian3,
	
	pub camera_projection: Mat4,
	
}

// Maybe move the keyframe data to its own component?
// One that says this entity belongs to a keyframe?
// Could avoid creating new entities when creating keyframes, just add the keyframe component
#[derive(Component)]
#[require(PointIndex)]
pub struct ProjectedTracker {
	pub keyframe: Entity,
	pub tracked2d_pos: Vec2,
}


fn create_slam_keyframe(
	mut commands: Commands,
	slam_camera_query: Query<(Entity, &GlobalTransform, &Camera), With<SlamCamera>>,
	track2d_query: Query<&Track2D>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
) {
	
	if keyboard_input.just_pressed(KeyCode::KeyE) == false {
		return;
	}
	
	
	let mut cameras_in_created_keyframes: Vec<(Entity, Entity)> = Vec::new();
	
	for (slam_camera, globaltransform, slam_camera_camera3d) in slam_camera_query {
		
		let Some(viewport_size) = slam_camera_camera3d.logical_viewport_size() else {continue;};
		
		
		
		let slam_keyframe_entity = commands.spawn((
			Transform {
				translation: globaltransform.translation(),
				rotation: globaltransform.rotation(),
				..default()
			},
			SlamKeyframe {
				position_uncertainty: Gaussian3 {mean: Vec3::ZERO, covariance: Mat3::IDENTITY},
				rotation_uncertainty: Gaussian3 {mean: Vec3::ZERO, covariance: Mat3::IDENTITY},
				
				camera_projection: slam_camera_camera3d.computed.clip_from_view,
				
			},
		)).id();
		cameras_in_created_keyframes.push((slam_camera, slam_keyframe_entity));
		
		println!("Created SLAM keyframe");
	}
	for track2d in track2d_query {
		
		let mut owned_keyframe: Option<Entity> = None;
		let Some(owned_camera) = track2d.owned_camera else {
			println!("Cannot add Track2D to keyframe, no owned camera");
			continue;
		};
		for (slam_camera, slam_keyframe) in &cameras_in_created_keyframes {
			
			if &owned_camera == slam_camera {
				owned_keyframe = Some(slam_keyframe.clone());
				break;
			}
			
		}
		
		if let Some(found_owned_keyframe) = owned_keyframe {
			
			let x = track2d.viewport_position;
			println!("Tracked pos: {x}");
			
			let projected_tracker = commands.spawn((
				ProjectedTracker {
					keyframe: found_owned_keyframe,
					tracked2d_pos: track2d.viewport_position,
				},
			)).id();
			
			commands.entity(found_owned_keyframe).add_child(projected_tracker);
			
			
		}
		else {
			println!("Failed to create ProjectedTracker, owner keyframe not found");
		}
		
		
	}
	
}


pub struct SlamPlugin;
impl Plugin for SlamPlugin {
	fn build(&self, app: &mut App) {
		
		app.insert_resource(PointIndexCounter(0));
		
		app.add_systems(Update, assign_tracker_index);
		app.add_systems(Update, apply_worldtrackpoint_tracks.after(assign_tracker_index));
		app.add_systems(Update, create_slam_keyframe);
		
	}
}



