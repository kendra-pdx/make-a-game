use bevy::prelude::*;

pub fn mouse_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    camera_query
        .get_single()
        .map_or(None, |(camera, camera_transform)| {
            windows
                .single()
                .cursor_position()
                .and_then(|cursor_position| {
                    camera.viewport_to_world_2d(camera_transform, cursor_position)
                })
        })
}

pub fn create_sprite(handle: &Handle<Image>) -> SpriteBundle {
    SpriteBundle {
        texture: handle.clone(),
        ..default()
    }
}
