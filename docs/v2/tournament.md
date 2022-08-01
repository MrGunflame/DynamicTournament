# `/v2/tournament`

The `/v2/tournament` endpoint exposes the avaliable tournaments. Note that unlike the name suggests, this endpoint returns a list of tournaments.

## Types

### Tournament

| Field        | Type   | Description |
| ------------ | ------ | --- |
| id           | u64    | The unqiue id of the tournament |
| name         | string | The name of the tournament |
| description  | string | A more detailed description of the tournament |
| date         | string | The starting datetime of the tournament in RFC3339 UTC format |
| bracket_type | string | The type of the bracket. Either `"single_elimination"` or `"double_elimination"` for single elimination or double elimination brackets respectively |
| entrants     | object | An object containing a key `players` with a []Player value of a key `teams` with a []Team value. |

### Team

A team of players playing in a tournament.

| Field   | Type     | Description                   |
| ------- | -------- | ----------------------------- |
| name    | string   | The name of the team          |
| players | []Player | A list of players in the team |

### Player

A single player in a tournament. Depending on the tournament type, players can play themselves or as a part of a team.

| Field | Type | Description |
| ----- | ---- | ----------- |
| name | string | The name of the player |
| role | string | The role of the player when part of a team. `"unknown"` for *Unknown*, `"roamer"` for *Roamer*, `"teamfighter"` for *Teamfighter*, `"duelist"` for *Duelist*, `"support"` for *Support* |
| rating | u64 &#124; `null` | The rating of the player or `null` to disable. Defaults to `null`. |

## GET `/v2/tournament`

Returns a list of tournaments.

### Response Body

Returns a list of tournaments with the following fields:

| Field        | Type   | Description                                                     |
| ------------ | ------ | --------------------------------------------------------------- |
| id           | u64    | The unique id of the tournament                                 |
| name         | string | The name of the tournament                                      |
| date         | string | The starting datetime of the tournament in RFC3339 UTC format   |
| bracket_type | string | The type of the bracket. Either `"single_elimination"` or `"double_elimination"` for single elimination or double elimination brackets respectively |
| entrants     | u64    | The number of entrants playing in the tournament                |

## GET `/v2/tournament/:id`

Returns the tournament with the requested `id`.

### Response Body

Returns a complete [Tournament](#tournament).

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## POST `/v2/tournament`

Creates a new tournament. Returns the `id` of the newly created tournament on success.

### Request Body

| Field | Type | Description |
| - | - | - |
| name | string | The name of the tournament |
| description | string | A detailed description of the tournament |
| date | string | The starting datetime of the tournament in RFC3339 |
| bracket_type | string |
| entrants | object | An object containing a key `players` with a [][Player](#player) value or key `teams` with a [][Team](#team) value. |

### Request Headers

- `Authorization`: `Bearer <TOKEN>`

### Errors

- `400 Bad Request`: Returned if the request body is malformed or is missing required fields.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
