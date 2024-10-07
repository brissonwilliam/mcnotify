# Notify minecraft server logs activty to Discord. 

This is a personal learning project for my first steps in Rust

## Usage 
`export DISCORD_URL=${your url here}; ./mcnotify --path mcserver/logs/latest.log`

### TODOs
- make http calls async
- cache the regexes
- better error handling (might have too many unwrap() that could panic)
- implement more server known log events (deaths, gamemode etc)
