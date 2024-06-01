create table one (
    id bigint primary key generated always as identity,
    created_at timestamp with time zone not null default now(),
    value text
);
alter publication supabase_realtime add table one;

create table two (
    id bigint primary key generated always as identity,
    created_at timestamp with time zone not null default now(),
    value jsonb
);
alter publication supabase_realtime add table two;

create table three (
    id bigint primary key generated always as identity,
    created_at timestamp with time zone not null default now(),
    one_id bigint references one(id)
);
alter publication supabase_realtime add table three;

create table four (
    id bigint primary key generated always as identity,
    created_at timestamp with time zone not null default now(),
    value uuid default gen_random_uuid()
);
alter publication supabase_realtime add table four;

create table five (
    id bigint primary key generated always as identity,
    created_at timestamp with time zone not null default now(),
    value boolean default false
);
alter publication supabase_realtime add table five;
