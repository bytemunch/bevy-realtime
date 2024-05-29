# Bevy Realtime

## Compatibility

| bevy         | bevy-realtime |
| ------------ | ------------- |
| 0.13.x       | 0.1.0         |

## LICENSE

MIT or Apache 2

## Local example environment

A sample local Supabase is provided. It runs on the non-standard port 64321, and the browser
interface is available at http://localhost:64323. Change directory to `/supabase` and run `supabase
start` to launch it. To target it when running examples, export the environment variable
`BEVY_REALTIME_LOCAL_ANON_KEY` and set it to the value returned by `supabase status`.

This should allow you to perform changes to the database which can be detected by `bevy_realtime`
examples.
