use std::{collections::HashMap, time::Duration};

use bevy::{ecs::system::SystemId, prelude::*, time::common_conditions::on_timer};
use bevy_realtime::{
    channel::ChannelBuilder,
    client::ConnectError,
    message::payload::PresenceConfig,
    presence::{PrescenceTrack, PresenceEvent, PresenceState},
    BevyChannelBuilder, BuildChannel, Channel, Client, RealtimePlugin,
};

#[derive(Resource, Deref)]
struct PresenceJoinCallback(SystemId<(String, PresenceState, PresenceState)>);

#[derive(Resource, Deref)]
struct PresenceStateCallback(pub SystemId<PresenceState>);

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
            (get_presence_state.run_if(on_timer(Duration::from_secs(1))),),
        );

    app.run()
}

fn setup(world: &mut World) {
    world.spawn(Camera2dBundle::default());

    let build_channel_callback = world.register_system(build_channel_callback);
    let connect_callback = world.register_system(connect_callback);

    let client = world.resource::<Client>();
    let _ = client.connect(connect_callback);
    client.channel(build_channel_callback).unwrap();

    let get_presence_state_callback = world.register_system(get_presence_state_callback);
    world.insert_resource(PresenceStateCallback(get_presence_state_callback));

    let presence_join_callback = world.register_system(presence_join_callback);
    world.insert_resource(PresenceJoinCallback(presence_join_callback));
}

fn build_channel_callback(
    mut channel_builder: In<ChannelBuilder>,
    mut commands: Commands,
    presence_join_callback: Res<PresenceJoinCallback>,
) {
    channel_builder
        .topic("test")
        .set_presence_config(PresenceConfig {
            key: Some("TestPresKey".into()),
        })
        .on_presence(PresenceEvent::Join, **presence_join_callback);

    let mut channel = commands.spawn(BevyChannelBuilder(channel_builder.0));

    let mut payload = HashMap::new();

    payload.insert("Location".into(), "UK".into());

    channel.insert(PrescenceTrack { payload });

    channel.insert(BuildChannel);
}

fn get_presence_state(channel: Query<&Channel>, callback: Res<PresenceStateCallback>) {
    for c in channel.iter() {
        c.presence_state(**callback).unwrap();
    }
}

fn get_presence_state_callback(state: In<PresenceState>) {
    println!("State got! {:?}", *state);
}

fn presence_join_callback(In((id, state, joins)): In<(String, PresenceState, PresenceState)>) {
    println!("{}|{:?}|{:?}", id, state, joins);
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
