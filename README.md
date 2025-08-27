# rssify - Transform some website I like too RSS feeds

- lwn paid articles
- CISA KEV release

## Usage

### Using nix

```nix
nix develop  # to get the dev dependencies
nix build
nix run -- /tmp/rss/
```

### Using podman

Use the `justfile` to run commands:

```bash
just init  # Will init the database, and build the app image
just run   # Will run the lwn-sub-snoozer to update the database and the RSS file
```

## TODO

- [ ] Add volume to the db to store it if it crashes
- [ ] Add tests
