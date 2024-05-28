use bevy::{ecs::system::SystemId, prelude::*};
use bevy_gotrue::{just_logged_in, AuthCreds, AuthPlugin, Client as AuthClient};
use bevy_http_client::HttpClientPlugin;
use bevy_realtime::{
    channel::ChannelBuilder,
    message::{
        payload::{PostgresChangesEvent, PostgresChangesPayload},
        postgres_change_filter::PostgresChangeFilter,
    },
    BevyChannelBuilder, BuildChannel, Client as RealtimeClient, RealtimePlugin,
};

#[derive(Resource, Deref)]
struct OnChangeCallback(pub SystemId<PostgresChangesPayload>);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((
            HttpClientPlugin,
            RealtimePlugin::new(
                "http://127.0.0.1:54321/realtime/v1".into(),
                std::env::var("SUPABASE_LOCAL_ANON_KEY").unwrap(),
            ),
            AuthPlugin {
                endpoint: "http://127.0.0.1:54321/auth/v1".into(),
            },
        ))
        .add_systems(Startup, (setup, sign_in))
        .add_systems(Update, signed_in.run_if(just_logged_in));

    app.run()
}

fn setup(world: &mut World) {
    world.spawn(Camera2dBundle::default());

    let callback = world.register_system(build_channel_callback);
    let client = world.resource::<RealtimeClient>();
    client.channel(callback).unwrap();

    let on_change_callback = world.register_system(on_change_callback);
    world.insert_resource(OnChangeCallback(on_change_callback));
}

fn sign_in(mut commands: Commands, auth: Res<AuthClient>) {
    auth.sign_in(
        &mut commands,
        AuthCreds {
            id: "test@example.com".into(),
            password: "password".into(),
        },
    );
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

fn signed_in(client: Res<RealtimeClient>, auth: Res<AuthClient>) {
    client
        .set_access_token(auth.access_token.clone().unwrap())
        .unwrap();
}

fn on_change_callback(input: In<PostgresChangesPayload>) {
    println!("Change got! {:?}", *input);
}
