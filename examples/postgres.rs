use bevy::{ecs::system::SystemId, prelude::*};
use bevy_realtime::{
    channel::ChannelBuilder,
    client::ConnectError,
    message::{
        payload::{PostgresChangesEvent, PostgresChangesPayload},
        postgres_change_filter::PostgresChangeFilter,
    },
    BevyChannelBuilder, BuildChannel, Client, RealtimePlugin,
};

#[derive(Resource, Deref)]
struct OnChangeCallback(pub SystemId<PostgresChangesPayload>);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((RealtimePlugin::new(
            "http://127.0.0.1:54321/realtime/v1".into(),
            std::env::var("SUPABASE_LOCAL_ANON_KEY").unwrap(),
        ),))
        .add_systems(Startup, (setup,));

    app.run()
}

fn setup(world: &mut World) {
    world.spawn(Camera2dBundle::default());
    let connect_callback = world.register_system(connect_callback);
    let build_channel_callback = world.register_system(build_channel_callback);

    let client = world.resource::<Client>();
    let _ = client.connect(connect_callback);
    client.channel(build_channel_callback).unwrap();

    let on_change_callback = world.register_system(on_change_callback);
    world.insert_resource(OnChangeCallback(on_change_callback));
}

fn build_channel_callback(
    mut channel_builder: In<ChannelBuilder>,
    mut commands: Commands,
    on_change_callback: Res<OnChangeCallback>,
) {
    channel_builder.topic("test").on_postgres_change(
        PostgresChangesEvent::All,
        PostgresChangeFilter {
            schema: "public".into(),
            table: Some("todos".into()),
            filter: None,
        },
        **on_change_callback,
    );

    let mut channel = commands.spawn(BevyChannelBuilder(channel_builder.0));

    channel.insert(BuildChannel);
}

fn on_change_callback(input: In<PostgresChangesPayload>) {
    println!("Change got! {:?}", *input);
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
