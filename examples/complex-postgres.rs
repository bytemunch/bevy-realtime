use bevy::{ecs::system::SystemId, prelude::*};
use bevy_realtime::{
    channel::ChannelBuilder,
    message::{
        payload::{PostgresChangesEvent, PostgresChangesPayload},
        postgres_change_filter::PostgresChangeFilter,
    },
    BevyChannelBuilder, BuildChannel, Client, RealtimePlugin,
};

#[derive(Resource, Deref)]
struct OnOneCallback(pub SystemId<PostgresChangesPayload>);
#[derive(Resource, Deref)]
struct OnTwoCallback(pub SystemId<PostgresChangesPayload>);
#[derive(Resource, Deref)]
struct OnThreeCallback(pub SystemId<PostgresChangesPayload>);
#[derive(Resource, Deref)]
struct OnFourCallback(pub SystemId<PostgresChangesPayload>);
#[derive(Resource, Deref)]
struct OnFiveCallback(pub SystemId<PostgresChangesPayload>);

#[derive(Component)]
struct TableInsertMessage;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(RealtimePlugin::new(
            "http://127.0.0.1:64321/realtime/v1".into(),
            std::env::var("BEVY_REALTIME_LOCAL_ANON_KEY").unwrap(),
        ))
        .add_systems(Startup, setup);

    app.run()
}

fn setup(world: &mut World) {
    world.spawn(Camera2dBundle::default());

    let build_channel_callback = world.register_system(build_channel);
    let client = world.resource::<Client>();
    client.channel(build_channel_callback).unwrap();

    let on_one = world.register_system(read_one);
    let on_two = world.register_system(read_two);
    let on_three = world.register_system(read_three);
    let on_four = world.register_system(read_four);
    let on_five = world.register_system(read_five);
    world.insert_resource(OnOneCallback(on_one));
    world.insert_resource(OnTwoCallback(on_two));
    world.insert_resource(OnThreeCallback(on_three));
    world.insert_resource(OnFourCallback(on_four));
    world.insert_resource(OnFiveCallback(on_five));
}

fn build_channel(
    mut channel_builder: In<ChannelBuilder>,
    mut commands: Commands,
    on_one: Res<OnOneCallback>,
    on_two: Res<OnTwoCallback>,
    on_three: Res<OnThreeCallback>,
    on_four: Res<OnFourCallback>,
    on_five: Res<OnFiveCallback>,
) {
    channel_builder
        .topic("table_inserts")
        .on_postgres_change(
            PostgresChangesEvent::All,
            PostgresChangeFilter {
                schema: "public".into(),
                table: Some("one".into()),
                filter: None,
            },
            **on_one,
        )
        .on_postgres_change(
            PostgresChangesEvent::All,
            PostgresChangeFilter {
                schema: "public".into(),
                table: Some("two".into()),
                filter: None,
            },
            **on_two,
        )
        .on_postgres_change(
            PostgresChangesEvent::All,
            PostgresChangeFilter {
                schema: "public".into(),
                table: Some("three".into()),
                filter: None,
            },
            **on_three,
        )
        .on_postgres_change(
            PostgresChangesEvent::All,
            PostgresChangeFilter {
                schema: "public".into(),
                table: Some("four".into()),
                filter: None,
            },
            **on_four,
        )
        .on_postgres_change(
            PostgresChangesEvent::All,
            PostgresChangeFilter {
                schema: "public".into(),
                table: Some("five".into()),
                filter: None,
            },
            **on_five,
        );

    let mut channel = commands.spawn(BevyChannelBuilder(channel_builder.0));

    channel.insert(BuildChannel);
    info!("::: insert into tables one, two, three, four, or five :::");
}

// The In<PostgresChangesPayload> argument must be passed first here, but otherwise we can use the
// usual system commands and queries.
fn read_one(
    payload: In<PostgresChangesPayload>,
    mut commands: Commands,
    insert_messages: Query<Entity, With<TableInsertMessage>>,
) {
    info!(payload.data.table);
    for e in insert_messages.iter() {
        commands.entity(e).despawn_recursive();
    }

    if let Some(record) = payload.data.record.clone() {
        commands.spawn((
            TextBundle::from_section(
                format!(
                    "table {} insert: {}",
                    payload.data.table,
                    record.get("value").unwrap()
                ),
                TextStyle {
                    color: Color::ALICE_BLUE,
                    font_size: 50.,
                    ..default()
                },
            ),
            TableInsertMessage,
        ));
    }
}

fn read_two(
    payload: In<PostgresChangesPayload>,
    mut commands: Commands,
    insert_messages: Query<Entity, With<TableInsertMessage>>,
) {
    info!(payload.data.table);
    for e in insert_messages.iter() {
        commands.entity(e).despawn_recursive();
    }

    if let Some(record) = payload.data.record.clone() {
        commands.spawn((
            TextBundle::from_section(
                format!(
                    "table {} insert: {}",
                    payload.data.table,
                    record.get("value").unwrap()
                ),
                TextStyle {
                    color: Color::ORANGE_RED,
                    font_size: 50.,
                    ..default()
                },
            ),
            TableInsertMessage,
        ));
    }
}

fn read_three(
    payload: In<PostgresChangesPayload>,
    mut commands: Commands,
    insert_messages: Query<Entity, With<TableInsertMessage>>,
) {
    info!(payload.data.table);
    for e in insert_messages.iter() {
        commands.entity(e).despawn_recursive();
    }

    commands.spawn((
        TextBundle::from_section(
            "table three insert",
            TextStyle {
                color: Color::SALMON,
                font_size: 50.,
                ..default()
            },
        ),
        TableInsertMessage,
    ));
}

fn read_four(
    payload: In<PostgresChangesPayload>,
    mut commands: Commands,
    insert_messages: Query<Entity, With<TableInsertMessage>>,
) {
    info!(payload.data.table);
    for e in insert_messages.iter() {
        commands.entity(e).despawn_recursive();
    }

    commands.spawn((
        TextBundle::from_section(
            "table four insert",
            TextStyle {
                color: Color::DARK_GREEN,
                font_size: 50.,
                ..default()
            },
        ),
        TableInsertMessage,
    ));
}

fn read_five(
    payload: In<PostgresChangesPayload>,
    mut commands: Commands,
    insert_messages: Query<Entity, With<TableInsertMessage>>,
) {
    info!(payload.data.table);
    for e in insert_messages.iter() {
        commands.entity(e).despawn_recursive();
    }

    commands.spawn((
        TextBundle::from_section(
            "table five insert",
            TextStyle {
                color: Color::AQUAMARINE,
                font_size: 50.,
                ..default()
            },
        ),
        TableInsertMessage,
    ));
}
