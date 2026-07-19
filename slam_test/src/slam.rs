use bevy::prelude::*;

#[derive(Component)]
pub struct SLAMCamera;

#[derive(Component)]
pub struct MapCamera;

#[derive(Component)]
#[require(Transform)]
pub struct WorldTrackPoint;

