# rssify - Transform some website I like too RSS feeds

- lwn paid articles
- CISA KEV release

## Usage

Use the `justfile` to run commands:

```bash
just init  # Will init the database, and build the app image
just run   # Will run the lwn-sub-snoozer to update the database and the RSS file
```

## Nix setup

For my server, I have the following setup:

```nix
systemd.timers."rssify-update" = {
  wantedBy = [ "timers.target" ];
    timerConfig = {
      OnBootSec = "5m";
      OnUnitActiveSec = "12h";
      Unit = "rssify-update.service";
    };
};

systemd.services."rssify-update" = {
  script = ''
    ${pkgs.rssify}/bin/echo "Hello World"
  '';
  serviceConfig = {
    Type = "oneshot";
    User = "root";
  };
};
```

## TODO

- [ ] Nix service with timer
- [ ] Better path managment, with env variable in Dockerfile
- [ ] Clean repo
- [ ] Add volume to the db to store it if it crashes
- [ ] Add tests
