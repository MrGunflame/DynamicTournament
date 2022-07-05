# /v3/tournaments/:id/entrants

This sub-endpoint is avaliable for all tournaments. It is used to create, delete or update the entrants of the tournament.

## Types

An entrant can be one of two:

### Player

| Field  | Type            | Description                                                                  |
| ------ | --------------- | ---------------------------------------------------------------------------- |
| id     | u64             | The unique identifier of the entrant.                                         |
| name   | string          | The name of the player.                                                      |
| rating | u64 &#124; null | The rating of the player or `null` to disable. Optional, defaults to `null`. |
| role   | u64             | The id of the role of the player. Optional, defaults to `0`. (See [Roles endpoint](roles.md) ) |

### Team

| Field   | Type                | Description                        |
| ------- | ------------------- | ---------------------------------- |
| id      | u64                 | The unique identifier of the entrant. |
| name    | string              | The name of the team.              |
| players | [Player](#player)[] | A list of all players in the team. |

## GET `/v3/tournaments/:id/entrants`

Returns all entrants in the tournament. This returns either a list of [Player](#player)s or [Team](#team)s depending on the `kind` field of the tournament.

### Response Body

Returns a either a list of [Player](#player)s or [Team](#team)s.

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## POST `/v3/tournaments/:id/entrants`

Creates a new entrant in the tournament with the given `id`. Please note that all players must have valid role ids for the tournament.

### Request Headers

- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains the new [Player](#player) or [Team](#team) with the following fields:

| Field  | Type            | Description                                                                  |
| ------ | --------------- | ---------------------------------------------------------------------------- |
| name   | string          | The name of the player.                                                      |
| rating | u64 &#124; null | The rating of the player or `null` to disable. Optional, defaults to `null`. |
| role   | u64             | The id of the role of the player. Optional, defaults to `0`. (See [Roles endpoint](roles.md) ) |

| Field   | Type                | Description                        |
| ------- | ------------------- | ---------------------------------- |
| name    | string              | The name of the team.              |
| players | [Player](#player)[] | A list of all players in the team. |

The request can also contain a list of [Player](#player)s or [Team](#team)s.

### Response Body

Returns the newly created entrant. If multiple entrants have been created, returns a list of the newly created entrants.

### Errors

- `400 Bad Request`: Returned if the request body is malformed, missing fields or the entrant type is not applicable for the type of the requested tournament.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## PATCH `/v3/tournaments/:id/entrants/:id`

Updates a number of fields of the entrant with the given `id`. Please note that all players must have valid role ids for the tournament.

| Field   | Note                                                               |
| ------- | ------------------------------------------------------------------ |
| id      | Cannot be updated. If sent, this field is always ignored.          |
| players | Updating the `players` field in a `Team` **replaces** all players. |

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains all fields to be updated.

### Response Body

Returns the update entrant.

### Errors

- `400 Bad Request`: Returned if the request body is malformed.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or entrant with the requested `id` does not exist

## DELETE `/v3/tournaments/:id/entrants/:id`

Deletes the entrant with the given `id`.

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Errors

- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or entrant with the requested `id` does not exist.
