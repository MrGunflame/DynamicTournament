# V3

## Api Endpoints

- [/v3/auth](auth.md)
- [/v3/systems](systems.md)
- [/v3/tournaments](tournaments.md)

## Authentication

Authentication for write-endpoints is provided via a [JWT](https://jwt.io) in the `Authorization` header. If the header is missing for a request a `401` error
is returned. The payload contains the following claims:

| Field | Type  | Description                                                        |
| ----- | ----- | ------------------------------------------------------------------ |
| sub   | u64   | The id of the user.                                                |
| iat   | u64   | The timestamp in seconds of the creation of this token.            |
| exp   | u64   | Expiration timestamp in seconds of this token.                     |
| nbf   | u64   | Timestamp in seconds before which the token may no be used yet.    |
| flags  | u8    | (Optional) A bitmap of optional features this token is capable of. |

**Note that the payload has the same stability guarantees as the rest of the REST API, i.e. new fields may be added in the future.**

## Request Headers

For read-only requests no request headers are required. For write requests additional headers may be required:

- `Authorization`: This header is always required when requesting write operations.
- `Content-Length`: This header is always required for requests with a request body. If not present the request will be rejected.
- `Content-Type`: This header is not required, but recommended for requests with a request body. The server will always assume the request body is `application/json`.

## Migration from V2

In v3 the primary Tournament struct used by the v2 version is split up and less hardcoded. It now only contains the `id`, `name`, `date`, `description` and `kind` fields. The `id`, `name`, `date` and `description` fields are identical to their v2 counterpart. In v2 the `entrants` field accepted **either** an `teams` or `players` field. In v3 the `kind` field now defines whether a tournament accepts teams or singular players. Any data about the entrants and bracket is split into separate endpoints.

The `/v3/tournaments/:id/entrants` endpoint now exclusively handles the entrants of the tournament. They are not provided when the tournament is first created. The type of entrant is identified automatically and does not need to be specified manually when creating a new entrant. Note that the entrant must match the `kind` definition of the tournament (i.e. creating a player in a `team` tournament will fail (and vice-versa)). The Team struct is exactly the same as in v2. The Player struct has one difference: the `role` field is now a `u64` and is an id for an Role provided by `/v3/tournaments/:id/roles`.

The `/v3/tournaments/:id/roles` endpoint handles player roles. They only consist out of an `id` and `name`. **The role with the id `0` is always "Unknown".** In v2 the roles were hardcoded into the api. Now you must manually create roles before being able to assign them to players in the tournament.

The `/v3/tournaments/:id/brackets` endpoint now handles creating brackets that should be rendered. This is a change from v2, where there was always exactly one bracket with all players generated automatically. Creating a new bracket now requires choosing a system using the `system` field. This was previously hardcoded in the `bracket_type` field of the tournament. In v3 systems are provided via the `/v3/systems` endpoint. The `entrants` field now allows to specify entrant ids (returned from `/v3/tournaments/:id/entrants`) of the entrants playing in the bracket.

The `/v2/auth` and `/v3/auth` endpoints behave identically. Also note that tokens are interchangable between api versions.
