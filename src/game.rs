use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_health_bar::{ProgressBar, ProgressBarBundle, ProgressBarPlugin};
use bevy::render::camera::ScalingMode;
use bevy_scroller::{
    Scroller, ScrollerBundle, ScrollerDirection, ScrollerPlugin, ScrollerSize, SingleSpriteGenerator
};
use bevy_tweening::{lens::*, *};
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::Rng;
use std::collections::VecDeque;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Plugins.
        app.add_plugins((
            PhysicsPlugins::default(),
            ProgressBarPlugin,
            ScrollerPlugin,
            TweeningPlugin,
            // Debugging...
            //PhysicsDebugPlugin::default(),
            //WorldInspectorPlugin::new()
        ));

        // App state.
        app.add_state::<AppState>();

        // Debug types.
        app.register_type::<Speed>();
        app.register_type::<EnemyDirection>();
        app.register_type::<EnemyHealth>();

        // Resources.
        app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)));
        app.insert_resource(Gravity(Vector::NEG_Y * 100.0 * 10.0));

        app.add_systems(Startup, spawn_health_bar);

        // MainMenu state sytems.
        app.add_systems(OnEnter(AppState::MainMenu),
            setup_main_menu
        );
        app.add_systems(Update, (
                action_main_menu,
                button_main_menu
            ).run_if(in_state(AppState::MainMenu))
        );
        app.add_systems(OnExit(AppState::MainMenu),
            despawn_screen::<OnMainMenuScreen>
        );

        // Credits state systems.
        app.add_systems(OnEnter(AppState::Credits),
            setup_credits
        );
        app.add_systems(Update, (
                action_credits,
                button_credits
            ).run_if(in_state(AppState::Credits))
        );
        app.add_systems(OnExit(AppState::Credits),
            despawn_screen::<OnCreditsScreen>
        );

        // InGame state systems.
        app.add_systems(OnEnter(AppState::InGame),
            (setup_game, setup_snow_and_projectiles, reset_health_bar).chain()
        );
        app.add_systems(Update, (
                anim_snow_fx,
                anim_enemy,
                move_enemy,
                anim_player,
                move_player,
                spawn_snow,
                move_snow,
                spawn_enemy_projectiles,
                move_enemy_projectiles,
                collide_snow_with_player,
                collide_snow_with_enemy,
                update_health_bar,
                remove_snow,
                remove_enemy_projectiles
            ).chain()
            .run_if(in_state(AppState::InGame))
        );
        app.add_systems(OnExit(AppState::InGame), (
                hide_health_bar,
                despawn_screen::<OnInGameScreen>
            )
        );

        // Win state systems.
        app.add_systems(OnEnter(AppState::Win),
            setup_win_screen
        );
        app.add_systems(Update, (
                action_credits,
                button_credits
            ).run_if(in_state(AppState::Win))
        );
        app.add_systems(OnExit(AppState::Win),
            despawn_screen::<OnWinGameScreen>
        );
        // Lose state systems.
        app.add_systems(OnEnter(AppState::Lose),
            setup_lose_screen
        );
        app.add_systems(Update, (
                action_credits,
                button_credits
            ).run_if(in_state(AppState::Lose))
        );
        app.add_systems(OnExit(AppState::Lose),
            despawn_screen::<OnLoseGameScreen>
        );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    MainMenu,
    Credits,
    #[default]
    InGame,
    Win,
    Lose
}

// MainMenu data and functions...

// Text and button styling.
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag for entities added to the main menu.
#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnCreditsScreen;

#[derive(Component)]
struct OnInGameScreen;

#[derive(Component)]
struct OnWinGameScreen;

#[derive(Component)]
struct OnLoseGameScreen;

// Tag to mark the selected button.
#[derive(Component)]
struct SelectedButton;

// All possible button actions for the main menu.
#[derive(Component)]
enum MainMenuButtonActions {
    Start,
    Credits
}
#[derive(Component)]
enum OtherButtonActions {
    Back
}

fn setup_main_menu(
    mut commands: Commands
) {
    // Define the base button styles.
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    // Set up the button layout using nodes.
    commands.spawn((
        OnMainMenuScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                min_height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }
    ))
    .with_children(|parent| {
        // Title text.
        parent.spawn(TextBundle::from_section(
            "Lots of Snow",
            button_text_style.clone()
        ));
        // Start button.
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            MainMenuButtonActions::Start,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Start",
                button_text_style.clone()
            ));
        });
        // Credits button.
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            MainMenuButtonActions::Credits
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Credits",
                button_text_style.clone()
            ));
        });
    });
    // Create camera to view the menu.
    commands.spawn(Camera2dBundle::default()).insert(OnMainMenuScreen);
}

fn action_main_menu(
    interaction_query: Query<(&Interaction, &MainMenuButtonActions), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                MainMenuButtonActions::Start => {
                    app_state.set(AppState::InGame);
                },
                MainMenuButtonActions::Credits => {
                    app_state.set(AppState::Credits);
                }
            }
        }
    }
}

fn action_credits(
    interaction_query: Query<(&Interaction, &OtherButtonActions), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>
) {
    for (interaction, button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_action {
                OtherButtonActions::Back => {
                    app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

fn button_main_menu(
    mut query: Query<(&Interaction , &mut BackgroundColor, Option<&SelectedButton>), (Changed<Interaction>, With<Button>)>
) {
    for (interaction, mut color, selected) in &mut query {
        *color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn button_credits(
    mut query: Query<(&Interaction , &mut BackgroundColor, Option<&SelectedButton>), (Changed<Interaction>, With<Button>)>
) {
    for (interaction, mut color, selected) in &mut query {
        *color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn setup_credits(
    mut commands: Commands
) {
    // Define the base button styles.
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    // Set up the button layout using nodes.
    commands.spawn((
        OnCreditsScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }
    ))
    .with_children(|parent| {
        // Title text.
        parent.spawn(TextBundle::from_section(
            "Credits",
            button_text_style.clone()
        ));
        // Start button.
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            OtherButtonActions::Back,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Back",
                button_text_style.clone()
            ));
        });
    });
    // Create camera to view the menu.
    commands.spawn(Camera2dBundle::default()).insert(OnCreditsScreen);
}

// InGame data and functions...

#[derive(Resource)]
pub struct ScrollerImages(Vec<Handle<Image>>);

#[derive(Component)]
struct PlayerCapsule;

#[derive(Component)]
struct PlayerSprite;

#[derive(Component)]
struct EnemyCapsule;

#[derive(Component)]
struct EnemyProjectile;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct EnemyHealth (f32);

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct EnemyDirection(f32);

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Speed(f32);

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct SnowTile;

#[derive(Resource)]
struct SnowConfig {
    // How often the snow should spawn.
    timer: Timer,
}

#[derive(Resource)]
struct ProjectileConfig {
    // How often the projectiles should spawn.
    timer: Timer,
}


// Define the collision layers
#[derive(PhysicsLayer)]
enum Layer {
    Player,
    Enemy,
    EnemyProjectile,
    Wall,
    Ground,
    Snow
}

#[derive(Component)]
struct ToDelete;

#[derive(Component)]
struct DidDamage;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct Healthbar;
#[derive(Component)]
struct Shadowbar;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn spawn_health_bar(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        Name::new("EnemyHealthBarShadow"),
        Shadowbar,
        ProgressBarBundle {
            progresss_bar: ProgressBar {
                value: 100.0,
                max_value: 100.0,
                ..default()
            },
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("healthbar.png"),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    color: Color::rgb(0.1, 0.1, 0.1),
                    ..default()
                },
                transform: Transform::from_xyz(-125.0, -110.0, 400.0),
                ..default()
            },
            ..default()
        }
    ));

    // Spawn the health bar.
    commands.spawn((
        Name::new("EnemyHealthBar"),
        Healthbar,
        ProgressBarBundle {
            progresss_bar: ProgressBar {
                value: 100.0,
                max_value: 100.0,
                ..default()
            },
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("healthbar.png"),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                transform: Transform::from_xyz(-125.0, -110.0, 500.0),
                ..default()
            },
            ..default()
        }
    ));
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    // Health bar text with drop shadow.
    commands.spawn((
        OnInGameScreen,
        Name::new("EnemyHpText"),
        TextBundle::from_section(
            "Enemy HP",
            TextStyle {
                font_size: 16.0,
                color: Color::rgb(0.0, 0.0, 0.0),
                ..default()
            }
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(6.0),
            right: Val::Px(496.0),
            ..default()
        })
    ));
    commands.spawn((
        OnInGameScreen,
        Name::new("EnemyHpText"),
        TextBundle::from_section(
            "Enemy HP",
            TextStyle {
                font_size: 16.0,
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            }
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(7.0),
            right: Val::Px(497.0),
            ..default()
        })
    ));

    commands.spawn((
        OnInGameScreen,
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 1.0,
                scaling_mode: ScalingMode::Fixed { width: 320.0, height: 240.0 },
                near: -1000.0,
                far: 1000.0,
                ..default()
            },
            ..default()
        }
    ));

    // Spawn the bg layers.
    let bg_layers = (0..=4)
        .map(|i| format!("parallax/{i}.png"))
        .collect::<VecDeque<String>>();
    let bg_layer_handles = bg_layers
        .iter()
        .map(|image_path| asset_server.load(image_path))
        .collect::<Vec<Handle<Image>>>();
    commands.insert_resource(ScrollerImages(bg_layer_handles));
    let bg_sizes = [
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
        Vec2::new(320.0, 240.0),
    ];
    let scroller_speed_min = 0.2;
    let scroller_speed_step = 0.2;
    bg_sizes.into_iter().enumerate().for_each(|(i, size)| {
        commands.spawn((
            OnInGameScreen,
            ScrollerSize {
                size: Vec2::new(320.0, 240.0)
            },
            ScrollerBundle {
                scroller: Scroller {
                    speed: scroller_speed_min + i as f32 * scroller_speed_step,
                    direction: ScrollerDirection::Backward,
                    render_layer: Some(1),
                    ..default()
                },
                generator: SingleSpriteGenerator {
                    path: format!("parallax/{i}.png"),
                    size,
                },
                spatial: SpatialBundle::from_transform(Transform::from_translation(
                    Vec3::new(0.0, 0.0, 1.0 + i as f32)
                )),
                ..default()
            },
        ));
    });

    // Spawn the snow at ~80 z-index.
    let texture_handle = asset_server.load("snow_fx_sheet_4_half_opacity.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(320.0, 240.0), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indices = AnimationIndices{first: 0, last: 3};
    commands.spawn((
        OnInGameScreen,
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_xyz(0.0, 0.0, 80.0),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.10, TimerMode::Repeating))
    ));

    // Spawn the player with its physics, sprite, and tween animations.
    // The sprite is a child of the capsule/SpatialBundle so it can
    // rotate independently.
    commands.spawn((
        OnInGameScreen,
        Name::new("PlayerEntity"),
        PlayerCapsule,
        SpatialBundle {
            visibility: Visibility::Inherited,
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::capsule(32.0, 16.0),
        CollisionLayers::new([Layer::Player],
            [Layer::Ground, Layer::Wall, Layer::Snow, Layer::EnemyProjectile]),
        LockedAxes::new().lock_rotation(),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Mass(1.0),
        Speed(32.0)
    ))
    .with_children(|parent| {
        let base_duration_ms: u64 = 500;
        let tween = Tween::new(
            EaseFunction::ElasticInOut,
            std::time::Duration::from_millis(base_duration_ms),
            TransformRotationLens {
                start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 9.),
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 9.),
            }
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::Repeat);

        parent.spawn((
            PlayerSprite,
            SpriteBundle {
                texture: asset_server.load("player_pixel_1.png"),
                ..default()
            },
            Animator::new(tween)));
    });

    // Spawn the enemy with its physics, sprite, and tween animations.
    // The sprite is a child of the capsule/SpatialBundle so it can
    // rotate independently.
    commands.spawn((
        OnInGameScreen,
        Name::new("EnemyEntity"),
        EnemyCapsule,
        EnemyDirection(1.0),
        EnemyHealth(100.0),
        SpriteBundle {
            texture: asset_server.load("enemy.png"),
            transform: Transform::from_xyz(64.0, 96.0, 100.0)
                .with_scale(Vec3::new(2.0, 2.0, 1.0)),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(8.0),
        CollisionLayers::new([Layer::Enemy],
            [Layer::Snow]),
        Sensor,
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(0.0),
        Mass(0.0),
        Speed(32.0)
    ));

    // Spawn the ground.
    commands.spawn((
        OnInGameScreen,
        Name::new("Ground"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.95, 0.95, 1.0),
                custom_size: Some(Vec2::new(640.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -104.0, 200.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(640.0, 32.0),
        CollisionLayers::new([Layer::Ground],
            [Layer::Player, Layer::Snow]),
    ));

    // Spawn the walls.
    commands.spawn((
        OnInGameScreen,
        Name::new("WallEntityL"),
        Wall,
        SpatialBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(-192.0, -56.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(64.0, 64.0),
        CollisionLayers::new([Layer::Wall],
            [Layer::Player]),
    ));
    commands.spawn((
        OnInGameScreen,
        Name::new("WallEntityR"),
        Wall,
        SpatialBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(192.0, -56.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(64.0, 64.0),
        CollisionLayers::new([Layer::Wall],
            [Layer::Player]),
    ));
}

fn anim_snow_fx (
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlasSprite)>
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn anim_enemy(
    mut query: Query<(&mut AngularVelocity, &EnemyDirection), With<EnemyCapsule>>
) {
    for (mut angular_velocity, dir) in query.iter_mut() {
        angular_velocity.0 += 1.0 * dir.0;

        // Apply friction.
        let enemy_friction = 0.8;
        angular_velocity.0 *= enemy_friction;
    }
}

fn move_enemy(
    mut enemy: Query<(&mut LinearVelocity, &mut EnemyDirection, &Transform), With<EnemyCapsule>>
) {
    for (mut linear_vel, mut dir, xform) in enemy.iter_mut() {
        // Flip the movement direction when x bounds are hit.
        if xform.translation.x < -130.0 && dir.0 < 0.0
        {
            dir.0 *= -1.0;
        }
        if xform.translation.x > 130.0 && dir.0 > 0.0
        {
            dir.0 *= -1.0;
        }

        let enemy_speed = 40.0;
        let enemy_friction = 0.8;

        // Apply velocity.
        linear_vel.x += dir.0 * enemy_speed;
        // Apply friction.
        linear_vel.x *= enemy_friction;
    }
}

fn anim_player(
    keys: Res<Input<KeyCode>>,
    mut animators: Query<&mut Animator<Transform>, With<PlayerSprite>>
) {
    let hold_action = keys.any_pressed([
        KeyCode::Space,
        KeyCode::Z, KeyCode::X, KeyCode::C,
        KeyCode::Return,
        KeyCode::Tab,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight]);
    for mut animator in animators.iter_mut() {
        let base_duration_ms: u64 = 500;
        let norm_speed = animator.speed() < 1.5f32;
        let fast_speed = animator.speed() > 1.5f32;

        if norm_speed && hold_action {
            let tween = Tween::new(
                EaseFunction::ElasticInOut,
                std::time::Duration::from_millis(base_duration_ms),
                TransformRotationLens {
                    // These angles lean the sprite forward. (Towards screen right.)
                    start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 4.),
                    end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 12.),
                }
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::Repeat);
            animator.set_tweenable(tween);
            animator.set_speed(2.0);
        } else if fast_speed && !hold_action{
            let tween = Tween::new(
                EaseFunction::ElasticInOut,
                std::time::Duration::from_millis(base_duration_ms),
                TransformRotationLens {
                    start: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::PI / 9.),
                    end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 9.),
                }
            )
            .with_repeat_count(RepeatCount::Infinite)
            .with_repeat_strategy(RepeatStrategy::Repeat);
            animator.set_tweenable(tween);
            animator.set_speed(1.0);
        }
    }
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    mut players: Query<(&mut LinearVelocity, &Speed), With<PlayerCapsule>>
) {
    for (mut linear_vel, player_speed) in &mut players {
        // Only move left and right.
        let mut direction = Vec2::ZERO;
        let left = keys.any_pressed([KeyCode::A, KeyCode::Left]);
        let right = keys.any_pressed([KeyCode::D, KeyCode::Right]);
        if left {
            direction += Vec2::new(-1.0, 0.0);
        }
        if right {
            direction += Vec2::new(1.0, 0.0);
        }
        direction.y = 0.0;

        // Set the velocity.
        linear_vel.x += direction.normalize_or_zero().x * player_speed.0;

        // Apply friction.
        linear_vel.x *= 0.8;
    }
}

fn setup_snow_and_projectiles(
    mut commands: Commands
) {
    commands.insert_resource(
        SnowConfig {
            // Create the repeating timer.
            timer: Timer::new(std::time::Duration::from_millis(500), TimerMode::Repeating)
        }
    );
    commands.insert_resource(
        ProjectileConfig {
            // Create the repeating timer.
            timer: Timer::new(std::time::Duration::from_millis(1200), TimerMode::Repeating)
        }
    );
}

fn spawn_snow(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<SnowConfig>,
    snow: Query<Entity, With<SnowTile>>
) {
    // Tick the snow timer.
    config.timer.tick(time.delta());

    const MAX_SNOW: usize = 100;
    let snow_count = snow.iter().count();
    if config.timer.finished() && snow_count < MAX_SNOW {
        // Pick a random snow sprite each time.
        let mut rng = rand::thread_rng();
        let sprites = vec!["snow_1.png", "snow_2.png"];
        let sprite_idx: usize = rng.gen_range(0..sprites.len());
        let path = sprites[sprite_idx];
        // Spawn the snow sprite with its physics components.
        commands.spawn((
            OnInGameScreen,
            Name::new("SnowTile"),
            SnowTile,
            SpriteBundle {
                texture: asset_server.load(path),
                // Just above the player in z-order.
                transform: Transform::from_xyz(192.0, 0.0, 300.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(32.0, 32.0),
            CollisionLayers::new([Layer::Snow],
                [Layer::Player, Layer::Ground, Layer::Enemy, Layer::EnemyProjectile]),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            Mass(100.0),
            Speed(24.0)
        ));
    }
}

fn move_snow(
    mut snow: Query<(&mut LinearVelocity, &Speed), With<SnowTile>>
) {
    for (mut linear_vel, speed) in &mut snow {
        linear_vel.x += Vec2::new(-1.0, 0.0).normalize_or_zero().x * speed.0;
    }
}

fn spawn_enemy_projectiles(
    mut commands: Commands,
    enemy_query: Query<&Transform, With<EnemyCapsule>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<ProjectileConfig>
) {
    // Tick the snow timer.
    config.timer.tick(time.delta());

    if config.timer.finished() {
        // Spawn the projectile sprite with its physics components.
        commands.spawn((
            OnInGameScreen,
            Name::new("EnemyProjectile"),
            EnemyProjectile,
            SpriteBundle {
                texture: asset_server.load("enemy_projectile.png"),
                transform: enemy_query.single().clone(),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(8.0, 8.0),
            CollisionLayers::new([Layer::EnemyProjectile],
                [Layer::Player, Layer::Snow]),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            Mass(10.0),
            LinearVelocity(Vec2::new(0.0, 200.0)),
            Speed(10.0)
        ));
    }
}

fn move_enemy_projectiles(
    mut projectiles: Query<(&mut AngularVelocity, &mut LinearVelocity, &Speed), With<EnemyProjectile>>
) {
    for (mut ang_vel, mut lin_vel, speed) in &mut projectiles {
        let mut rng = rand::thread_rng();
        let av: f32 = rng.gen_range(-1.0..1.0);
        let x_vel: f32 = rng.gen_range(-3.0..3.0);
        let friction: f32 = 0.8;

        ang_vel.0 += av;
        ang_vel.0 *= friction;

        lin_vel.x += x_vel * friction;
        lin_vel.y -= speed.0 * friction;
    }
}

fn collide_snow_with_player(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    player: Query<Entity, With<PlayerCapsule>>,
    mut collisions: Query<(Entity, &mut LinearVelocity, &CollidingEntities), With<SnowTile>>
) {
    let force = 160.0;
    let hold_action = keys.any_pressed([
        KeyCode::Space,
        KeyCode::Z, KeyCode::X, KeyCode::C,
        KeyCode::Return,
        KeyCode::Tab,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight]);
    for (entity, mut linear_vel, colliding_entities) in &mut collisions {
        if hold_action && colliding_entities.contains(&player.single())
        {
            linear_vel.x += Vec2::new(1.0, 0.0).x * force;
            linear_vel.y += Vec2::new(0.0, 2.0).y * force;

            // Add the toDelete component.
            commands.entity(entity).insert(ToDelete);
        }
    }
}

fn collide_snow_with_enemy(
    mut commands: Commands,
    enemy: Query<Entity, With<EnemyCapsule>>,
    mut enemy_health: Query<&mut EnemyHealth>,
    mut collisions: Query<(Entity, &CollidingEntities), (With<SnowTile>, Without<DidDamage>)>
) {
    for (entity, colliding_entities) in &mut collisions {
        if colliding_entities.contains(&enemy.single())
        {
            let mut rng = rand::thread_rng();
            let damage: f32 = rng.gen_range(0.01..5.0);
            // Debugging... let damage: f32 = rng.gen_range(10.0..20.0);
            enemy_health.single_mut().0 -= damage;

            // Mark the snow tile as used.
            commands.entity(entity).insert(DidDamage);
        }
    }
}

fn update_health_bar(
    mut query: Query<&mut ProgressBar, With<Healthbar>>,
    health_query: Query<&EnemyHealth>,
    dt: Res<Time>,
    mut app_state: ResMut<NextState<AppState>>
) {
    for mut healthbar in query.iter_mut() {
        let health = health_query.single().0;
        if health < healthbar.value {
            healthbar.value -= (healthbar.max_value - health) * dt.delta_seconds();
        }
        if healthbar.value <= 0.0 {
            app_state.set(AppState::Win);
        }
    }
}

fn remove_snow(
    mut commands: Commands,
    snow: Query<(Entity, &Transform), (With<SnowTile>, With<ToDelete>)>
) {
    for (entity, transform) in &snow {
        if transform.translation.x < -130.0 || transform.translation.x > 250.0 ||
            transform.translation.y < -100.0 || transform.translation.y > 150.0
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn remove_enemy_projectiles(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<EnemyProjectile>>
) {
    for (entity, transform) in &projectiles {
        if transform.translation.y < -100.0
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_screen<T: Component>(
    mut commands: Commands,
    to_despawn: Query<Entity, With<T>>
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_win_screen(
    mut commands: Commands
) {
    // Define the base button styles.
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    // Set up the button layout using nodes.
    commands.spawn((
        OnWinGameScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }
    ))
    .with_children(|parent| {
        // Title text.
        parent.spawn(TextBundle::from_section(
            "You Win!",
            button_text_style.clone()
        ));
        // Start button.
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            OtherButtonActions::Back,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Return to Main Menu",
                button_text_style.clone()
            ));
        });
    });
    // Create camera to view the menu.
    commands.spawn(Camera2dBundle::default()).insert(OnWinGameScreen);
}

fn setup_lose_screen(
    mut commands: Commands
) {
    // Define the base button styles.
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    // Set up the button layout using nodes.
    commands.spawn((
        OnLoseGameScreen,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }
    ))
    .with_children(|parent| {
        // Title text.
        parent.spawn(TextBundle::from_section(
            "You Lose!",
            button_text_style.clone()
        ));
        // Start button.
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            OtherButtonActions::Back,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Return to Main Menu",
                button_text_style.clone()
            ));
        });
    });
    // Create camera to view the menu.
    commands.spawn(Camera2dBundle::default()).insert(OnLoseGameScreen);
}

fn reset_health_bar(
    mut hbar_query: Query<(&mut Visibility, &mut ProgressBar), (With<Healthbar>, Without<Shadowbar>)>,
    mut sbar_query: Query<(&mut Visibility, &mut ProgressBar), (With<Shadowbar>, Without<Healthbar>)>
) {
    for (mut v, mut hbar) in &mut hbar_query {
        *v = Visibility::Visible;
        hbar.value = 100.0;
    }

    for (mut v, mut sbar) in &mut sbar_query {
        *v = Visibility::Visible;
        sbar.value = 100.0;
    }
}

fn hide_health_bar(
    mut hbar: Query<&mut Visibility, (With<Healthbar>, Without<Shadowbar>)>,
    mut sbar: Query<&mut Visibility, (With<Shadowbar>, Without<Healthbar>)>,
) {
    for mut b in &mut hbar {
        *b = Visibility::Hidden;
    }
    for mut b in &mut sbar {
        *b = Visibility::Hidden;
    }
}