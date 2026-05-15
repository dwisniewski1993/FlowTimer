use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowLevel};
use std::time::{Duration, Instant};
use rodio::{OutputStream, Sink, source::SineWave, Source};

const ROBOTO_BYTES: &[u8] = include_bytes!("Roboto-SemiBold.ttf");

#[derive(Resource)]
struct EmbeddedFont(Handle<Font>);

#[derive(Resource)]
struct TimerState {
    start: Option<Instant>,
    triggered_10: bool,
    triggered_5: bool,
    handle: rodio::OutputStreamHandle,
}

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct StartButton;

fn main() {
    let (_stream, handle) = OutputStream::try_default().expect("Brak audio");

    App::new()
        .insert_resource(TimerState {
            start: None,
            triggered_10: false,
            triggered_5: false,
            handle,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FlowTimer".into(),
                transparent: true,
                // Wymagane na macOS dla prawdziwej przezroczystości okna
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                window_level: WindowLevel::AlwaysOnTop,
                decorations: false,
                resizable: false,
                resolution: (100u32, 36u32).into(),
                focused: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgba(0., 0., 0., 0.)))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_start_click, update_timer, drag_window))
        .run();
}

fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
) {
    commands.spawn(Camera2d);

    let embedded = fonts.add(
        Font::try_from_bytes(ROBOTO_BYTES.to_vec()).expect("Nie można załadować czcionki"),
    );
    commands.insert_resource(EmbeddedFont(embedded.clone()));

    commands.spawn((
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(36.0),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        StartButton,
    ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Start"),
                TextFont {
                    font: embedded.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_start_click(
    mut commands: Commands,
    interaction: Query<(Entity, &Interaction), (With<Button>, With<StartButton>)>,
    mut timer: ResMut<TimerState>,
    font: Res<EmbeddedFont>,
) {
    for (entity, interaction) in &interaction {
        if *interaction == Interaction::Pressed {
            timer.start = Some(Instant::now());
            timer.triggered_10 = false;
            timer.triggered_5 = false;

            commands.entity(entity).despawn();

            commands.spawn((
                Text::new("02:00"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                TimerText,
            ));
        }
    }
}

fn update_timer(
    mut timer: ResMut<TimerState>,
    mut query: Query<(&mut Text, &mut TextColor), With<TimerText>>,
) {
    let Some(start_time) = timer.start else { return };

    let elapsed = start_time.elapsed().as_secs();

    if elapsed >= 120 {
        timer.start = Some(Instant::now());
        timer.triggered_10 = false;
        timer.triggered_5 = false;

        for (mut text, mut color) in &mut query {
            text.0 = "02:00".to_string();
            *color = TextColor(Color::WHITE);
        }
        return;
    }

    let remaining = 120 - elapsed;
    let display = format!("{:02}:{:02}", remaining / 60, remaining % 60);

    for (mut text, _) in &mut query {
        text.0 = display.clone();
    }

    if remaining == 10 && !timer.triggered_10 {
        play_beep(&timer.handle, 440.0, 800);
        timer.triggered_10 = true;
        for (_, mut color) in &mut query {
            *color = TextColor(Color::srgb(1.0, 1.0, 0.0));
        }
    }

    if remaining == 5 && !timer.triggered_5 {
        play_beep(&timer.handle, 880.0, 800);
        timer.triggered_5 = true;
        for (_, mut color) in &mut query {
            *color = TextColor(Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

fn play_beep(handle: &rodio::OutputStreamHandle, freq: f32, dur_ms: u64) {
    if let Ok(sink) = Sink::try_new(handle) {
        let source = SineWave::new(freq)
            .take_duration(Duration::from_millis(dur_ms))
            .amplify(0.8);
        sink.append(source);
        sink.detach();
    }
}

fn drag_window(
    buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = windows.single_mut() {
            window.start_drag_move();
        }
    }
}