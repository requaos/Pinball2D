use std::borrow::Borrow;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude as lyon;
use bevy_rapier2d::prelude::*;

pub struct PinsPlugin;

impl Plugin for PinsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_pins.after("launcher").label("pins"))
            .add_system(handle_pin_events)
            .add_system(respawn_pin_to_toggle_color);
    }
}

#[derive(Component)]
struct Pin {
    timestamp_last_hit: f64,
    position: Vec2,
    radius: f32,
}

fn spawn_pins(mut commands: Commands) {
    let pins_pos: [Vec2; 8] = [
        Vec2::new(
            crate::PIXELS_PER_METER * -0.20,
            crate::PIXELS_PER_METER * 0.35,
        ),
        Vec2::new(
            crate::PIXELS_PER_METER * 0.20,
            crate::PIXELS_PER_METER * 0.35,
        ),
        Vec2::new(crate::PIXELS_PER_METER * 0.15, crate::PIXELS_PER_METER * 0.2),
        Vec2::new(crate::PIXELS_PER_METER * -0.15, crate::PIXELS_PER_METER * 0.2),
        Vec2::new(crate::PIXELS_PER_METER * 0.09, crate::PIXELS_PER_METER * 0.8),
        Vec2::new(crate::PIXELS_PER_METER * -0.15, crate::PIXELS_PER_METER * 0.8),
        Vec2::new(crate::PIXELS_PER_METER * 0.17, crate::PIXELS_PER_METER * - 0.01),
        Vec2::new(crate::PIXELS_PER_METER * -0.20, crate::PIXELS_PER_METER * - 0.01),
    ];

    for i in 0..pins_pos.len() {
        let pin_pos = pins_pos[i];
        let mut size: Option<f32> = None;
        if vec![2,3].contains(i.borrow()) {
            size = Some(crate::PIXELS_PER_METER * 0.04);
        }
        if vec![0,1].contains(i.borrow()) {
            size = Some(crate::PIXELS_PER_METER * 0.07);
        }

        spawn_single_pin(&mut commands, pin_pos, None, size);
    }
}

fn spawn_single_pin(commands: &mut Commands, position: Vec2, timestamp_last_hit: Option<f64>, radius: Option<f32>) {
    let shape_pin = lyon::shapes::Circle {
        radius: radius.unwrap_or(crate::PIXELS_PER_METER * 0.05),
        center: Vec2::ZERO,
    };

    let temp_timestamp_last_hit = timestamp_last_hit.unwrap_or(0.0);

    let mut color = Color::GREEN;
    if temp_timestamp_last_hit == 0.0 {
        color = Color::TEAL;
    }

    commands
        .spawn(lyon::GeometryBuilder::build_as(
            &shape_pin,
            lyon::DrawMode::Outlined {
                fill_mode: lyon::FillMode::color(Color::BLACK),
                outline_mode: lyon::StrokeMode::new(color, 2.0),
            },
            Transform::default(),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::ball(shape_pin.radius))
        .insert(Transform::from_xyz(position.x, position.y, 0.0))
        .insert(Restitution::coefficient(2.3))
        .insert(Pin {
            timestamp_last_hit: temp_timestamp_last_hit,
            position: position,
            radius: shape_pin.radius,
        });
}

fn respawn_pin_to_toggle_color(
    mut query: Query<(Entity, &Pin), With<Pin>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, pin) in query.iter_mut() {
        let diff = time.raw_elapsed_seconds_f64() - pin.timestamp_last_hit;
        if pin.timestamp_last_hit > 0.0 && diff > 3.0 {
            //Color have been toggled for more than a second so respawn
            let pos = pin.position;
            commands.entity(entity).despawn();
            spawn_single_pin(&mut commands, pos, None, Some(pin.radius));
        }
    }
}

fn handle_pin_events(
    query: Query<(Entity, &Pin), With<Pin>>,
    time: Res<Time>,
    mut contact_events: EventReader<CollisionEvent>,
    mut commands: Commands,
) {
    for contact_event in contact_events.iter() {
        for (entity, pin) in query.iter() {
            if let CollisionEvent::Started(h1, h2, _event_flag) = contact_event {
                if h1 == &entity || h2 == &entity {
                    //Respawn to change color
                    let pos = pin.position;
                    let timestamp_last_hit = time.raw_elapsed_seconds_f64();
                    commands.entity(entity).despawn();
                    spawn_single_pin(&mut commands, pos, Some(timestamp_last_hit), Some(pin.radius));
                }
            }
        }
    }
}
