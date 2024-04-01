use crate::helpers::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use derive_more::Constructor;
use rand::prelude::*;

use std::{collections::HashMap, f32::consts::PI, sync::OnceLock};

#[derive(Constructor)]
pub struct Physics {
    enabled: bool,
    debug: bool,
}

impl Plugin for Physics {
    fn build(&self, app: &mut App) {
        if self.enabled {
            app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
                .add_systems(Startup, setup_resources)
                .add_systems(Startup, setup_fixed)
                .add_systems(Startup, setup_scoreboard)
                .add_systems(Update, on_collision)
                .add_systems(Update, on_click)
                .add_systems(Update, on_reset)
                .add_systems(Update, update_scoreboard)
                .add_systems(Update, cull);

            if self.debug {
                app.add_plugins(RapierDebugRenderPlugin::default());
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Texture {
    Brick,
    Hammer,
}

#[derive(Constructor)]
struct TextureInfo {
    path: &'static str,
}

#[derive(Component, Debug, PartialEq, Eq)]
enum Object {
    Ground,
    Hammer,
    Brick,
}

#[derive(Component, Debug)]
struct Sprite;

#[derive(Component, Debug)]
struct ScoreText;

#[derive(Component, Debug)]
struct Resettable;

#[derive(Debug, Default, PartialEq, Eq)]
enum GameState {
    #[default]
    NewGame,
    Started,
    GameOver,
}

#[derive(Resource, Default, Debug)]
struct Score {
    value: i32,
    hammers_created: i32,
    bricks_created: i32,
    bricks_broken: i32,
    state: GameState,
    high: i32,
}

impl Score {
    fn hammered(&mut self) {
        self.value = (self.value - 1).max(0);
        self.hammers_created += 1;
        self.state = GameState::Started;
    }

    fn bricked(&mut self) {
        self.bricks_created += 1;
    }

    fn brick_broke(&mut self) {
        self.value += 1;
        self.bricks_broken += 1;
        if self.bricks_broken == self.bricks_created {
            self.state = GameState::GameOver;
        }
    }

    fn reset(&mut self) {
        self.high = self.high.max(self.value);
        self.value = 0;
        self.hammers_created = 0;
        self.bricks_created = 0;
        self.bricks_broken = 0;
        self.state = GameState::NewGame;
    }
}

fn setup_resources(mut commands: Commands) {
    const GRAVITY_SCALE: f32 = 20.0;

    commands.insert_resource(Score::default());
    commands.insert_resource(RapierConfiguration {
        gravity: Vec2::Y * -9.81 * GRAVITY_SCALE,
        ..default()
    })
}

fn setup_scoreboard(mut commands: Commands) {
    commands
        .spawn(
            TextBundle::from_sections(vec![
                TextSection {
                    value: "Score: 0".into(),
                    style: TextStyle {
                        font_size: 18.0,
                        ..default()
                    },
                },
                TextSection {
                    value: "High Score: 0".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::GRAY,
                        ..default()
                    },
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(15.0),
                ..default()
            }),
        )
        .insert(ScoreText);
}

fn update_scoreboard(score: Res<Score>, mut text: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut text {
        text.sections[0].style.color = if score.state == GameState::GameOver {
            Color::WHITE
        } else if score.state == GameState::Started {
            Color::GREEN
        } else {
            Color::GRAY
        };

        let state = if score.state == GameState::GameOver {
            "GG"
        } else {
            "Score"
        };

        text.sections[0].value = format!(
            "{}: {} (bricks: {}, hammers: {})\n",
            state, score.value, score.bricks_created, score.hammers_created
        );
        text.sections[1].value = format!("High Score: {}", score.high);
    }
}

fn setup_fixed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(Collider::cuboid(450.0, 20.0))
        .insert(Object::Ground)
        .insert(Restitution::coefficient(0.2))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -300.0, 0.0)))
        .insert(InheritedVisibility::VISIBLE)
        .with_children(|ground| {
            ground.spawn(ColorMesh2dBundle {
                mesh: meshes
                    .add(Rectangle::from_corners(
                        Vec2::new(-450.0, -280.0),
                        Vec2::new(450.0, -320.0),
                    ))
                    .into(),
                material: materials.add(Color::GRAY),
                ..default()
            });
        });
}

fn on_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    objects: Query<&Object>,
    mut score: ResMut<Score>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Stopped(e1, e2, _) = collision_event {
            let n1 = objects.get(*e1);
            let n2 = objects.get(*e2);
            info!("collision: {n1:?} {n2:?}");

            for e in [e1, e2] {
                if let Ok(Object::Brick) = objects.get(*e) {
                    commands.entity(*e).despawn_recursive();
                    score.brick_broke();
                }
            }
        }
    }
}

fn on_click(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    assets: Res<AssetServer>,
    mut score: ResMut<Score>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: Query<&Window>,
) {
    // spawn a hammer
    if buttons.just_released(MouseButton::Left) && score.state != GameState::GameOver {
        if let Some(point) = mouse_position(&camera_query, &windows) {
            info!("click at: {:?}", point);
            let mut rng = rand::thread_rng();
            let hammer = get_texture(&assets, Texture::Hammer);
            commands
                .spawn(RigidBody::Dynamic)
                .insert(Object::Hammer)
                .insert(ColliderMassProperties::Density(2.0))
                .insert(ColliderMassProperties::Mass(2.0))
                .insert(Collider::cuboid(40.0, 40.0))
                .insert(Restitution::coefficient(0.8))
                .insert(TransformBundle::from(
                    Transform::from_xyz(point.x, point.y, 0.0)
                        .with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..2.0 * PI))),
                ))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(InheritedVisibility::VISIBLE)
                .insert(Resettable)
                .with_children(|collider| {
                    collider.spawn(create_sprite(&hammer)).insert(Sprite);
                });

            score.hammered();
        }
    }

    // spawn a brick
    if buttons.just_released(MouseButton::Right) && score.state == GameState::NewGame {
        if let Some(point) = mouse_position(&camera_query, &windows) {
            let brick = get_texture(&assets, Texture::Brick);
            commands
                .spawn(Collider::cuboid(30.0, 30.0))
                .insert(Object::Brick)
                .insert(ColliderMassProperties::Density(1.0))
                .insert(ColliderMassProperties::Mass(2.0))
                .insert(Restitution::coefficient(0.2))
                .insert(InheritedVisibility::VISIBLE)
                .insert(TransformBundle::from(Transform::from_xyz(
                    point.x, point.y, 0.0,
                )))
                .insert(Resettable)
                .with_children(|c| {
                    c.spawn(create_sprite(&brick));
                });

            score.bricked();
        }
    }
}

fn on_reset(
    mut commands: Commands,
    mut score: ResMut<Score>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    resettables: Query<Entity, With<Resettable>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        info!("resetting!");
        score.reset();
        for r in &resettables {
            commands.entity(r).despawn_recursive();
        }
    }
}

fn cull(mut commands: Commands, sprites: Query<(&Parent, &ViewVisibility), With<Sprite>>) {
    for (p, v) in &sprites {
        if !v.get() {
            info!("culling: {:?}", p);
            commands.entity(p.get()).despawn_recursive();
        }
    }
}

fn get_texture(assets: &Res<AssetServer>, texture: Texture) -> Handle<Image> {
    static TEXTURES: OnceLock<HashMap<Texture, TextureInfo>> = OnceLock::new();
    let textures = TEXTURES.get_or_init(|| {
        vec![
            (Texture::Brick, TextureInfo::new("brick.png")),
            (Texture::Hammer, TextureInfo::new("hammer.png")),
        ]
        .into_iter()
        .collect()
    });
    assets.load(textures[&texture].path)
}
