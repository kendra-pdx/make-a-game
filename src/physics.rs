use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use derive_more::Constructor;
use rand::prelude::*;

use std::f32::consts::PI;

#[derive(Constructor)]
pub struct Physics {
    enabled: bool,
}

#[derive(Resource)]
struct HammerTexture(Handle<Image>);

#[derive(Resource)]
struct BrickTexture(Handle<Image>);

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

#[derive(Resource, Default, Debug)]
struct Score {
    value: i32,
    hammers: i32,
    bricks: i32,
    started: bool,
    high: i32,
}

impl Score {
    fn hammered(&mut self) {
        self.value = (self.value - 1).max(0);
        self.hammers += 1;
        self.started = true;
    }

    fn bricked(&mut self) {
        self.bricks += 1;
    }

    fn brick_broke(&mut self) {
        self.value += 1;
    }

    fn reset(&mut self) {
        self.high = self.high.max(self.value);
        self.value = 0;
        self.hammers = 0;
        self.bricks = 0;
        self.started = false;
    }
}

impl Plugin for Physics {
    fn build(&self, app: &mut App) {
        if self.enabled {
            app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
                // .add_plugins(RapierDebugRenderPlugin::default())
                .add_systems(Startup, setup_resources)
                .add_systems(Startup, setup_fixed)
                .add_systems(Startup, setup_scoreboard)
                .add_systems(Update, on_collision)
                .add_systems(Update, on_click)
                .add_systems(Update, on_reset)
                .add_systems(Update, update_scoreboard)
                .add_systems(Update, cull);
        }
    }
}

fn setup_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(HammerTexture(asset_server.load("hammer.png")));
    commands.insert_resource(BrickTexture(asset_server.load("brick.png")));
    commands.insert_resource(Score::default());
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
        text.sections[0].style.color = if score.started {
            Color::GREEN
        } else {
            Color::GRAY
        };
        text.sections[0].value = format!(
            "Score: {} (bricks: {}, hammers: {})\n",
            score.value, score.bricks, score.hammers
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
        .insert(Restitution::coefficient(0.3))
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
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: Query<&Window>,
    mut commands: Commands,
    hammer: Res<HammerTexture>,
    brick: Res<BrickTexture>,
    mut score: ResMut<Score>,
) {
    // spawn a hammer
    if buttons.just_released(MouseButton::Left) {
        if let Some(point) = mouse_position(&camera_query, &windows) {
            info!("click at: {:?}", point);
            let mut rng = rand::thread_rng();

            commands
                .spawn(RigidBody::Dynamic)
                .insert(Object::Hammer)
                .insert(Collider::cuboid(40.0, 40.0))
                .insert(Restitution::coefficient(0.9))
                .insert(TransformBundle::from(
                    Transform::from_xyz(point.x, point.y, 0.0)
                        .with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..2.0 * PI))),
                ))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(InheritedVisibility::VISIBLE)
                .insert(Resettable)
                .with_children(|collider| {
                    collider.spawn(create_sprite(&hammer.0)).insert(Sprite);
                });

            score.hammered();
        }
    }

    // spawn a brick
    if buttons.just_released(MouseButton::Right) && !score.started {
        if let Some(point) = mouse_position(&camera_query, &windows) {
            commands
                .spawn(Collider::cuboid(30.0, 30.0))
                .insert(Object::Brick)
                .insert(Restitution::coefficient(0.5))
                .insert(InheritedVisibility::VISIBLE)
                .insert(TransformBundle::from(Transform::from_xyz(
                    point.x, point.y, 0.0,
                )))
                .insert(Resettable)
                .with_children(|c| {
                    c.spawn(create_sprite(&brick.0));
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

fn mouse_position(
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

fn create_sprite(handle: &Handle<Image>) -> SpriteBundle {
    SpriteBundle {
        texture: handle.clone(),
        ..default()
    }
}
