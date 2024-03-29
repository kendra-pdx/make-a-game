#![allow(unused)]

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use derive_more::Constructor;

#[derive(Constructor)]
pub struct BasicPlugin {
    enabled: bool
}

impl Plugin for BasicPlugin {
    fn build(&self, app: &mut App) {
        if self.enabled {
            app
                .add_systems(Startup, create_player)
                .add_systems(Update, (move_player, player_position).chain());
        }
    }
}



#[derive(Component, Constructor)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Name(String);

#[derive(Bundle)]
struct Sprite(MaterialMesh2dBundle<ColorMaterial>);

#[derive(Bundle, Constructor)]
struct PlayerBundle {
    marker: Player,
    position: Position,
    name: Name,
}

fn move_player(time: Res<Time>, mut query: Query<&mut Position, With<Player>>) {
    for mut position in &mut query {
        static R: f32 = 50.0;
        let t = time.elapsed_seconds_f64() as f32 % 20.0;
        position.x = t.sin() * R;
        position.y = t.cos() * R;
    }
}

fn player_position(mut query: Query<(&Position, &Name, &mut Transform), With<Player>>) {
    for (position, name, mut transform) in &mut query {
        println!("position of {}: {} {}", name.0, position.x, position.y);
        *transform = Transform::from_xyz(position.x, position.y, 0.0);
    }
}

fn create_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player = PlayerBundle::new(
        Player,
        Position::new(0., 0.),
        Name("kendra".into()),
    );

    let sprite = Sprite(MaterialMesh2dBundle {
        mesh: meshes
            .add(Circle { radius: 20.0 })
            .into(),
        material: materials.add(Color::BLUE),
        ..Default::default()
    });
    commands.spawn((sprite, player));
}
