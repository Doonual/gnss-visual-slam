use bevy::prelude::*;
use bevy::{
	color::palettes::basic::*,
	input_focus::{FocusCause, InputFocus},
	prelude::*,
};
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, prelude::*, render::render_resource::TextureFormat};
use std::ptr::null;




#[derive(Component)]
struct KeyboardControl {
	move_speed: f32,
	rotation_speed: f32,
}

#[derive(Component)]
struct SLAMCamera;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.init_resource::<InputFocus>();
	app.add_systems(Startup, scene.spawn());
	app.add_systems(Startup, setup);
	app.add_systems(Update, keyboard_control_system);
	app.add_systems(Update, button_system);
	app.run();
	
	
}

const NORMAL_BUTTON_COL: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON_COL: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON_COL: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
	mut input_focus: ResMut<InputFocus>,
	mut interaction_query: Query<(Entity, &Interaction, &mut BackgroundColor, &mut BorderColor, &mut Button, &Children), Changed<Interaction>,>,
	mut text_query: Query<&mut Text>,
) {
	for (entity, interaction, mut color, mut border, mut button, children) in &mut interaction_query {
		let mut text = text_query.get_mut(children[0]).unwrap();
		
		match *interaction {
			Interaction::Pressed => {
				input_focus.set(entity, FocusCause::Pressed);
				**text = "Press".to_string();
				*color = PRESSED_BUTTON_COL.into();
				*border= BorderColor::all(RED);
				button.set_changed(); // Mark this as changed. helps the accessibility system
			}
			Interaction::Hovered => {
				input_focus.set(entity, FocusCause::Pressed);
				**text = "Hover".to_string();
				*color = HOVERED_BUTTON_COL.into();
				*border= BorderColor::all(Color::WHITE);
				button.set_changed(); // Mark this as changed. helps the accessibility system
			}
			Interaction::None => {
				input_focus.clear();
				input_focus.set(entity, FocusCause::Pressed);
				**text = "Button".to_string();
				*color = NORMAL_BUTTON_COL.into();
				*border= BorderColor::all(Color::BLACK);
				button.set_changed(); // Mark this as changed. helps the accessibility system
			}
		}
	}
}


fn setup(mut commands: Commands, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>,) {
	
	
	let image = Image::new_target_texture(
		512,
		512,
		TextureFormat::Rgba8Unorm,
		Some(TextureFormat::Rgba8Unorm),
	);
	let image_handle = images.add(image);
	
	commands.spawn(
		(
			Camera3d::default(),
			Camera {
				order: -1,
				clear_color: Color::WHITE.into(),
				..default()
			},
			RenderTarget::Image(image_handle.clone().into()),
			//RenderLayers::layer(1),
			SLAMCamera,
			KeyboardControl{ move_speed: 10.0, rotation_speed: 0.05},
			Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
		)
	);
	
	
	commands.spawn(Camera2d);
	commands.spawn(button(&assets, image_handle));
	
}

fn button(asset_server: &AssetServer, image_handle: Handle<Image>) -> impl Bundle {
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
				Button,
				Node {
					width: px(150),
					height: px(65),
					border: UiRect::all(px(5)),
					// Horizontally center child text
					justify_content: JustifyContent::Center,
					// Vertically center child text
					align_items: AlignItems::Center,
					border_radius: BorderRadius::MAX,
					..default()
				},
				ImageNode {
					image: image_handle.clone(),
					..default()
				},
				BorderColor::all(Color::WHITE),
				BackgroundColor(Color::BLACK),
				children![(
					Text::new("Button"),
					TextFont {
						font: asset_server.load("fonts/UbuntuSansMonoNerdFontMono-Regular.ttf").into(),
						font_size: FontSize::Px(33.0),
						..default()
					},
					TextColor(Color::srgb(0.9, 0.9, 0.9)),
					TextShadow::default(),
				)]
			),
			
		]
	)
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
			//RenderLayers::layer(0).with(1) // Might not need render layer 0
		),
	]
	
}



