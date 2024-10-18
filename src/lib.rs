#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod channel;
pub mod client;
pub mod message;
pub mod presence;

use std::{thread::sleep, time::Duration};

use bevy::prelude::*;
use bevy_crossbeam_event::{CrossbeamEventApp, CrossbeamEventSender};
use channel::{
    BroadcastCallbackEvent, ChannelBuilder, ChannelManager, ChannelStateCallbackEvent,
    PostgresChangesCallbackEvent, PresenceStateCallbackEvent,
};
use client::{
    ChannelCallbackEvent, ClientBuilder, ClientManager, ConnectResultCallbackEvent,
    ConnectionState, NextMessageError,
};
use presence::PresenceCallbackEvent;

use crate::presence::{presence_untrack, update_presence_track};

#[derive(Resource, Deref)]
pub struct Client(pub ClientManager);

#[derive(Component, Deref, DerefMut)]
pub struct BevyChannelBuilder(pub ChannelBuilder);

#[derive(Component, Deref, DerefMut)]
pub struct Channel(pub ChannelManager);

#[derive(Component)]
pub struct BuildChannel;

fn build_channels(
    mut commands: Commands,
    mut q: Query<(Entity, &mut BevyChannelBuilder), With<BuildChannel>>,
    client: Res<Client>,
    presence_state_callback_event_sender: Res<CrossbeamEventSender<PresenceStateCallbackEvent>>,
    channel_state_callback_event_sender: Res<CrossbeamEventSender<ChannelStateCallbackEvent>>,
    broadcast_callback_event_sender: Res<CrossbeamEventSender<BroadcastCallbackEvent>>,
    presence_callback_event_sender: Res<CrossbeamEventSender<PresenceCallbackEvent>>,
    postgres_changes_callback_event_sender: Res<CrossbeamEventSender<PostgresChangesCallbackEvent>>,
) {
    for (e, c) in q.iter_mut() {
        commands.entity(e).remove::<BevyChannelBuilder>();

        let channel = c.build(
            &client.0,
            presence_state_callback_event_sender.clone(),
            channel_state_callback_event_sender.clone(),
            broadcast_callback_event_sender.clone(),
            presence_callback_event_sender.clone(),
            postgres_changes_callback_event_sender.clone(),
        );

        channel.subscribe().unwrap();
        commands.entity(e).insert(Channel(channel));
    }
}

pub struct RealtimePlugin {
    endpoint: String,
    apikey: String,
}

impl RealtimePlugin {
    pub fn new(endpoint: String, apikey: String) -> Self {
        Self { endpoint, apikey }
    }
}

impl Plugin for RealtimePlugin {
    fn build(&self, app: &mut App) {
        app.add_crossbeam_event::<ConnectionState>()
            .add_crossbeam_event::<ChannelCallbackEvent>()
            .add_crossbeam_event::<PresenceStateCallbackEvent>()
            .add_crossbeam_event::<ChannelStateCallbackEvent>()
            .add_crossbeam_event::<BroadcastCallbackEvent>()
            .add_crossbeam_event::<PresenceCallbackEvent>()
            .add_crossbeam_event::<PostgresChangesCallbackEvent>()
            .add_crossbeam_event::<ConnectResultCallbackEvent>()
            .add_systems(
                Update,
                (
                    ((
                        //
                        update_presence_track,
                        presence_untrack,
                        build_channels,
                    )
                        .chain()
                        .run_if(client_ready),),
                    run_callbacks,
                )
                    .chain(),
            );

        // TODO: Allow this to fail and be retried later at user request

        let mut client = ClientBuilder::new(self.endpoint.clone(), self.apikey.clone());
        client.reconnect_max_attempts(3);
        let mut client = client.build(
            app.world_mut()
                .resource::<CrossbeamEventSender<ChannelCallbackEvent>>()
                .clone(),
            app.world_mut()
                .resource::<CrossbeamEventSender<ConnectResultCallbackEvent>>()
                .clone(),
        );

        app.insert_resource(Client(ClientManager::new(&client)));

        // Start off thread client
        let _thread = std::thread::spawn(move || {
            loop {
                match client.step() {
                    Err(NextMessageError::WouldBlock) => {}
                    Ok(_) => {}
                    Err(_e) => {} //error!("{}", _e),
                }

                // TODO find a sane sleep value
                sleep(Duration::from_secs_f32(f32::MIN_POSITIVE));
            }
        });
    }
}

fn run_callbacks(
    mut commands: Commands,
    mut channel_evr: EventReader<ChannelCallbackEvent>,
    mut presence_state_evr: EventReader<PresenceStateCallbackEvent>,
    mut channel_state_evr: EventReader<ChannelStateCallbackEvent>,
    mut broadcast_evr: EventReader<BroadcastCallbackEvent>,
    mut presence_evr: EventReader<PresenceCallbackEvent>,
    mut postgres_evr: EventReader<PostgresChangesCallbackEvent>,
    mut connect_evr: EventReader<ConnectResultCallbackEvent>,
) {
    // TODO this is crying out for a macro lol
    for ev in channel_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }

    for ev in presence_state_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }

    for ev in channel_state_evr.read() {
        let (callback, input) = ev.0;
        commands.run_system_with_input(callback, input);
    }

    for ev in broadcast_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }

    for ev in presence_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }

    for ev in postgres_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }

    for ev in connect_evr.read() {
        let (callback, input) = ev.0.clone();
        commands.run_system_with_input(callback, input);
    }
}

pub fn client_ready(
    mut evr: EventReader<ConnectionState>,
    mut last_state: Local<ConnectionState>,
    client: Res<Client>,
    sender: Res<CrossbeamEventSender<ConnectionState>>,
) -> bool {
    client.connection_state(sender.clone()).unwrap_or(());

    for ev in evr.read() {
        *last_state = *ev;
    }

    *last_state == ConnectionState::Open
}
