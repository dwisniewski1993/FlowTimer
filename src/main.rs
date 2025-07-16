use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowLevel};
use bevy::winit::WinitWindows;
use bevy::ui::Interaction;
use std::time::{Duration, Instant};
use rodio::{OutputStream, Sink, source::SineWave, Source};
use winit::dpi::PhysicalPosition;

const ROBOTO_BYTES: &[u8] = include_bytes!("Roboto-SemiBold.ttf");

#[derive(Resource)]
struct EmbeddedFont(Handle<Font>);

#[derive(Resource)]
struct Timer {
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
        .insert_resource(Timer {
            start: None,
            triggered_10: false,
            triggered_5: false,
            handle,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FlowTimer".into(),
                transparent: true,
                window_level: WindowLevel::AlwaysOnTop,
                decorations: false,
                resizable: false,
                resolution: (100., 36.).into(),
                focused: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_start_click, update_timer, drag_window))
        .run();
}

fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
) {
    commands.spawn(Camera2dBundle::default());

    let embedded = fonts.add(Font::try_from_bytes(ROBOTO_BYTES.to_vec()).expect("Nie można załadować czcionki"));
    commands.insert_resource(EmbeddedFont(embedded.clone()));

    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(36.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.5, 0.5, 0.5).into(),
            ..default()
        },
        StartButton,
    ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Start",
                TextStyle {
                    font: embedded.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
        });
}

fn handle_start_click(
    mut commands: Commands,
    mut interaction: Query<(Entity, &Interaction), (With<Button>, With<StartButton>)>,
    mut timer: ResMut<Timer>,
    font: Res<EmbeddedFont>,
) {
    for (entity, interaction) in &mut interaction {
        if *interaction == Interaction::Pressed {
            timer.start = Some(Instant::now());
            timer.triggered_10 = false;
            timer.triggered_5 = false;

            commands.entity(entity).despawn_recursive();

            commands.spawn((
                TextBundle::from_section(
                    "02:00",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                )
                    .with_style(Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    }),
                TimerText,
            ));
        }
    }
}

fn update_timer(
    mut timer: ResMut<Timer>,
    mut query: Query<&mut Text, With<TimerText>>,
) {
    let Some(start_time) = timer.start else { return };

    let elapsed = start_time.elapsed().as_secs();

    if elapsed >= 120 {
        timer.start = Some(Instant::now());
        timer.triggered_10 = false;
        timer.triggered_5 = false;

        for mut text in &mut query {
            text.sections[0].style.color = Color::WHITE;
        }
    }

    let remaining = 120 - elapsed;
    let minutes = remaining / 60;
    let seconds = remaining % 60;
    let display = format!("{:02}:{:02}", minutes, seconds);

    for mut text in &mut query {
        text.sections[0].value = display.clone();
    }

    if remaining == 10 && !timer.triggered_10 {
        play_beep(&timer.handle, 440.0, 800);
        timer.triggered_10 = true;

        for mut text in &mut query {
            text.sections[0].style.color = Color::YELLOW;
        }
    }

    if remaining == 5 && !timer.triggered_5 {
        play_beep(&timer.handle, 880.0, 800);
        timer.triggered_5 = true;

        for mut text in &mut query {
            text.sections[0].style.color = Color::RED;
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
    buttons: Res<Input<MouseButton>>,
    mut cursor_events: EventReader<CursorMoved>,
    windows: NonSend<WinitWindows>,
    q: Query<Entity, With<PrimaryWindow>>,
    mut last: Local<Option<Vec2>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        *last = None;
        return;
    }

    if let Some(event) = cursor_events.iter().last() {
        let current = event.position;

        if let Some(prev) = *last {
            let delta = current - prev;

            if let Some(win) = windows.get_window(q.single()) {
                if let Ok(pos_win) = win.outer_position() {
                    let new_pos = PhysicalPosition::new(
                        pos_win.x + delta.x as i32,
                        pos_win.y + delta.y as i32,
                    );
                    win.set_outer_position(new_pos);
                }
            }
        }

        *last = Some(current);
    }
}
