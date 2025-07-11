use crate::constants::dojo::{MOVE_SELECTOR, SPAWN_SELECTOR};

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};
use dojo_bevy_plugin::{DojoEntityUpdated, DojoInitializedEvent, DojoResource, TokioRuntime};
use dojo_types::schema::Struct;
use starknet::core::types::{Call, Felt};
use std::collections::HashSet;
use torii_grpc_client::types::{Pagination, PaginationDirection, Query as ToriiQuery};

/// This event will be triggered every time the position is updated.
#[derive(Event)]
pub struct PositionUpdatedEvent(pub Position);

#[derive(Resource, Default)]
struct EntityTracker {
    existing_entities: HashSet<Felt>,
}

/// A very simple cube to represent the player.
#[derive(Component)]
pub struct Player {
    pub id: Felt,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EntityTracker>()
        .add_event::<PositionUpdatedEvent>()
        .add_systems(
            Update,
            (
                handle_keyboard_input,
                on_dojo_events,
                (update_player_position).after(on_dojo_events),
            ),
        );
}

/// This system is responsible for handling the keyboard input.
fn handle_keyboard_input(
    tokio: Res<TokioRuntime>,
    mut dojo: ResMut<DojoResource>,
    dojo_config: Res<super::DojoSystemState>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for event in keyboard_input_events.read() {
        let key_code = event.key_code;
        let is_pressed = event.state == ButtonState::Pressed;

        match key_code {
            KeyCode::Space if is_pressed => {
                info!("Spawning.");
                let calls = vec![Call {
                    to: dojo_config.config.action_address,
                    selector: SPAWN_SELECTOR,
                    calldata: vec![],
                }];

                dojo.queue_tx(&tokio, calls);
            }
            KeyCode::KeyS if is_pressed => {
                info!("Setting up Torii subscription.");
                dojo.subscribe_entities(&tokio, "position".to_string(), None);
            }
            KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
                if is_pressed =>
            {
                let direction = match key_code {
                    KeyCode::ArrowLeft => 0,
                    KeyCode::ArrowRight => 1,
                    KeyCode::ArrowUp => 2,
                    KeyCode::ArrowDown => 3,
                    _ => panic!("Invalid key code"),
                };

                let calls = vec![Call {
                    to: dojo_config.config.action_address,
                    selector: MOVE_SELECTOR,
                    calldata: vec![Felt::from(direction)],
                }];

                dojo.queue_tx(&tokio, calls);
            }
            _ => continue,
        }
    }
}

/// Updates the cube position by reacting to the dedicated event
/// for new position updates.
fn update_player_position(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut entity_tracker: ResMut<EntityTracker>,
    mut ev_position_updated: EventReader<PositionUpdatedEvent>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    for ev in ev_position_updated.read() {
        let Position { x, y, player } = ev.0;

        if !entity_tracker.existing_entities.contains(&player) {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
                Player { id: player },
                Transform::from_xyz(x as f32, y as f32, 0.0),
            ));

            entity_tracker.existing_entities.insert(player);
        } else {
            for (mut transform, player_comp) in query.iter_mut() {
                if player_comp.id == player {
                    transform.translation = Vec3::new(x as f32, y as f32, 0.0);
                }
            }
        }
    }
}

/// Reacts on Dojo events, which are emitted by the Dojo plugin.
///
/// Any `queue_retrieve_entities` or `subscribe_entities` call will trigger
/// the `DojoEntityUpdated` event.
fn on_dojo_events(
    mut dojo: ResMut<DojoResource>,
    tokio: Res<TokioRuntime>,
    mut ev_initialized: EventReader<DojoInitializedEvent>,
    mut ev_retrieve_entities: EventReader<DojoEntityUpdated>,
    mut ev_position_updated: EventWriter<PositionUpdatedEvent>,
) {
    for _ in ev_initialized.read() {
        info!("Dojo initialized.");

        // Initial fetch, which will make the Dojo plugin to send
        // the query Torii, and trigger the `DojoEntityUpdated` event.
        dojo.queue_retrieve_entities(
            &tokio,
            ToriiQuery {
                clause: None,
                pagination: Pagination {
                    limit: 100,
                    cursor: None,
                    direction: PaginationDirection::Forward,
                    order_by: vec![],
                },
                no_hashed_keys: false,
                models: vec![],
                historical: false,
            },
        );
    }

    // Since the deserialization of the models is project specific,
    // currently the way it is done is by emitting an event for each
    // models updates we are interested in.
    // This may become too much for a large number of models though.
    // Maybe the solution would be to generate a plugin via bindgen,
    // that registers all of this automatically.
    for ev in ev_retrieve_entities.read() {
        info!(entity_id = ?ev.entity_id, "Torii update");

        // Felt::ZERO is being emitted once, when the subcription is initialized.
        // We don't want to spawn a cube for this.
        if ev.entity_id == Felt::ZERO {
            continue;
        }

        for m in &ev.models {
            debug!("model: {:?}", &m);

            match m.name.as_str() {
                "di-Position" => {
                    ev_position_updated.write(PositionUpdatedEvent(m.into()));
                }
                name if name == "di-Moves".to_string() => {}
                _ => {
                    warn!("Model not handled: {:?}", m);
                }
            };
        }
    }
}

/// The position of the player in the game.
#[derive(Component, Debug)]
pub struct Position {
    pub player: Felt,
    pub x: u32,
    pub y: u32,
}

/// This implementation shows a manual way to map data from the Position model in Cairo.
/// Ideally, we want a binding generation to do that for us.
impl From<&Struct> for Position {
    fn from(struct_value: &Struct) -> Self {
        let player = struct_value
            .get("player")
            .unwrap()
            .as_primitive()
            .unwrap()
            .as_contract_address()
            .unwrap();
        let x = struct_value
            .get("x")
            .unwrap()
            .as_primitive()
            .unwrap()
            .as_u32()
            .unwrap();
        let y = struct_value
            .get("y")
            .unwrap()
            .as_primitive()
            .unwrap()
            .as_u32()
            .unwrap();

        Position { player, x, y }
    }
}
