# Dynamic Tournament Server

## Building

You need to have a stable rust toolchain installed (cargo).

1. Clone the repo: `git clone https://github.com/MrGunflame/DynamicTournament`
2. Build using make: `cd dynamic_tournament_server && make build`

The built binary is `target/release/dynamic-tournament-server`.

Building the docker image.

1. Clone the repo: `git clone https://github.com/MrGunflame/DynamicTournament`
2. Build using make: `cd dynamic_tournament_server && make docker`

The final docker image is tagged `dynamic-tournament-server`.

## Runtime Requirements

- MariaDB 12.2.7+ or MySQL 5.7.8+

## Configuration

The included [config.toml](https://github.com/MrGunflame/DynamicTournament/blob/master/dynamic-tournament-server/config.toml) file contains all
configuration options. Additionally all options can be overwritten using environment variables. If all options are set using environment variables the
config file still needs to be present currently.

### Global Options

| Option   | Type   | Possible Values                           | Environment Variable |
| -------- | ------ | ----------------------------------------- | -------------------- |
| loglevel | string | `error`, `warn`, `info`, `debug`, `trace` | DT_LOGLEVEL        |
| bind     | string | \<address\>:\<port\>                      | DT_BIND            |
  
### Database Options

| Option   | Type   | Possible Values | Environment Variable |
| -------- | ------ | --------------- | -------------------- |
| driver   | string | `mysql`         | DT_DB_DRIVER       |
| host     | string | any             | DT_DB_HOST         |
| port     | u16    | any             | DT_DB_PORT         |
| user     | string | any             | DT_DB_USER         |
| password | string | any             | DT_DB_PASSWORD     |
| database | string | any             | DT_DB_DATABASE     |

### User configuration

User configuration is required for mutating requests. Users are defined in [users.json](https://github.com/MrGunflame/DynamicTournament/blob/master/dynamic-tournament-server/users.json)
with every entry in the array being a user struct with username and password fields:

```
[
  {
    "username": "root",
    "password": "1234"
  }
]
```

**Note:** By default no users are configured.
