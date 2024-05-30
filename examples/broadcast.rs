use std::{collections::HashMap, time::Duration};

use bevy::{ecs::system::SystemId, prelude::*, time::common_conditions::on_timer};
use bevy_realtime::{
    channel::{ChannelBuilder, ChannelState},
    client::ConnectError,
    client_ready,
    message::payload::{BroadcastConfig, BroadcastPayload},
    BevyChannelBuilder, BuildChannel, Channel, Client, RealtimePlugin,
};
use serde_json::Value;

#[derive(Resource)]
struct MyBroadcastCallback(pub SystemId<HashMap<String, Value>>);

#[derive(Resource)]
struct ConnectCallback(pub SystemId<Result<(), ConnectError>>);

#[derive(Resource, Deref)]
struct ChannelStateCallback(pub SystemId<ChannelState>);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((RealtimePlugin::new(
            "http://127.0.0.1:54321/realtime/v1".into(),
            std::env::var("SUPABASE_LOCAL_ANON_KEY").unwrap(),
        ),))
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                (
                    ((send_every_second, get_channel_state)
                        .run_if(on_timer(Duration::from_secs(1))),)
                        .chain()
                        .run_if(client_ready),
                ),
                // Delayed connection
                connect
                    .run_if(on_timer(Duration::from_secs(3)))
                    .run_if(not(client_ready)),
            ),
        );

    app.run()
}

fn setup(world: &mut World) {
    info!("setup s1 ");

    world.spawn(Camera2dBundle::default());

    let build_channel_callback = world.register_system(build_channel_callback);
    let client = world.resource::<Client>();

    client.channel(build_channel_callback).unwrap();

    let test_callback = world.register_system(channel_state_callback);
    world.insert_resource(ChannelStateCallback(test_callback));
    let broadcast_callback = world.register_system(broadcast_callback);
    world.insert_resource(MyBroadcastCallback(broadcast_callback));
    let connect_callback = world.register_system(connect_callback);
    world.insert_resource(ConnectCallback(connect_callback));

    debug!("setup s1 finished");
}

fn build_channel_callback(
    mut channel_builder: In<ChannelBuilder>,
    mut commands: Commands,
    broadcast_callback: Res<MyBroadcastCallback>,
) {
    info!("channel setup s2 ");
    channel_builder
        .topic("test")
        .set_broadcast_config(BroadcastConfig {
            broadcast_self: true,
            ack: false,
        })
        .on_broadcast("test", broadcast_callback.0);

    let mut c = commands.spawn(BevyChannelBuilder(channel_builder.0));

    c.insert(BuildChannel);
    debug!("channel setup s2 finished");
}

fn get_channel_state(channel: Query<&Channel>, callback: Res<ChannelStateCallback>) {
    debug!("Get state...");
    for c in channel.iter() {
        c.channel_state(**callback).unwrap();
    }
}

fn channel_state_callback(state: In<ChannelState>) {
    info!("State got! {:?}", *state);
}

fn send_every_second(q_channel: Query<&Channel>) {
    let mut payload = HashMap::new();
    payload.insert("bevy?".into(), "bavy.".into());
    for c in q_channel.iter() {
        c.broadcast(BroadcastPayload {
            event: "test".into(),
            payload: payload.clone(),
            ..Default::default()
        })
        .unwrap();
    }
}

fn broadcast_callback(recv: In<HashMap<String, Value>>) {
    info!("GOT BC: {:?}", *recv);
}

fn connect(client: Res<Client>, callback: Res<ConnectCallback>) {
    let _ = client.connect(callback.0);
}

fn connect_callback(In(result): In<Result<(), ConnectError>>) {
    match result {
        Ok(()) => {
            info!("Connection is live!");
        }
        Err(e) => {
            error!("Connection failed! {:?}", e);
        }
    }
}
