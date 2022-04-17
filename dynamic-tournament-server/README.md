# Dynamic Tournament Server

## Building

### Requirements

- A stable rust toolchain (build only).
- MariaDB 12.2.7+ or MySQL 5.7.8+

### Building

- Clone the repo: `git clone https://github.com/MrGunflame/DynamicTournament`
- Build the server: `cargo build --bin dynamic-tournament-server --release`

## Configuration

The included [config.toml](https://github.com/MrGunflame/DynamicTournament/blob/master/dynamic-tournament-server/config.toml) file contains all
configuration options. Additionally all options can be overwritten using environment variables. If all options are set using environment variables the
config file still needs to be present currently.

### Global Options

| Option   | Type   | Possible Values                           | Environment Variable |
| -------- | ------ | ----------------------------------------- | -------------------- |
| loglevel | string | `error`, `warn`, `info`, `debug`, `trace` | DYNT_LOGLEVEL        |
| bind     | string | \<address\>:\<port\>                      | DYNT_BIND            |
  
### Database Options

| Option   | Type   | Possible Values | Environment Variable |
| -------- | ------ | --------------- | -------------------- |
| driver   | string | `mysql`         | DYNT_DB_DRIVER       |
| host     | string | any             | DYNT_DB_HOST         |
| port     | uint   | any             | DYNT_DB_PORT         |
| user     | string | any             | DYNT_DB_USER         |
| password | string | any             | DYNT_DB_PASSWORD     |
| database | string | any             | DYNT_DB_DATABASE     |

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
