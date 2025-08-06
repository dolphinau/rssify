## Usage

Use the `justfile` to run commands:

```bash
just init  # Will init the database, and build the app image
just run   # Will run the lwn-sub-snoozer to update the database and the RSS file
```

## TODO

- [ ] Nix service with timer
- [ ] Better path managment, with env variable in Dockerfile
- [ ] Clean repo
- [ ] Add volume to the db to store it if it crashes
- [ ] Add tests
