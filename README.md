# Notify minecraft server logs activity to Discord. 

This is a personal learning project for my first steps in Rust

![image](https://github.com/user-attachments/assets/8236ae77-ef8c-4554-a84b-2e31ea9efa1a)


## Usage 
`export DISCORD_URL=${your url here}; ./mcnotify --path ${mc_server_path}/logs/latest.log`

`--path` param must contain the path the file to watch
`DISCORD_URL` env var must be set to your discord webhook url

## Docker compose example
```
services:
  mcnotify:
    container_name: mcnotify
    restart: unless-stopped
    image: private.registry/mcnotify:latest
    command: --path /mclogs/latest.log
    volumes:
      - /your/path/to/mcserver/logs/:/mclogs/:ro
    environment:
      - DISCORD_URL=${your_url}
    labels:
     - "com.centurylinklabs.watchtower.enable=true"
```

This project builds on a private CircleCI org, and pushed image to a private docker image registry. You must build the app yourself.

### TODOs
- make http calls async
- cache the regexes
- better error handling (might have too many unwrap() that could panic)
- implement more server known log events (deaths, gamemode etc)
