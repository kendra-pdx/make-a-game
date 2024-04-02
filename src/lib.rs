mod basic;
mod helpers;
mod physics;

use basic::BasicPlugin;
use bevy::prelude::*;
use physics::Physics;
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;

fn create_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    gizmos.circle_2d(point, 10., Color::WHITE);
}

#[wasm_bindgen]
pub fn start() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins(DefaultPlugins)
        .add_plugins(BasicPlugin::new(false))
        .add_plugins(Physics::new(true, false))
        .add_systems(Startup, create_camera)
        .add_systems(Update, draw_cursor)
        .run();
}
