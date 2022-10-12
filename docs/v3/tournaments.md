# /v3/tournaments

The `/v3/tournaments` endpoint is used to create and manage Tournaments. Note that a tournament only provides basic fields. Entrants, brackets and roles are handled by the `/v3/tournaments/:id/entrants`, `/v3/tournaments/:id/brackets` and `/v3/tournaments/:id/roles` respectively. These sub-endpoints are avaliable under the tournament once it has been created.

## Types

### Tournament

| Field       | Type   | Description                                                                 |
| ----------- | ------ | --------------------------------------------------------------------------- |
| id          | u64    | A unique identifier for the tournament.                                     |
| name        | string | The name of the tournament.                                                 |
| description | string | The description of the tournament in an markdown format.                    |
| date        | string | Datetime of the tournament start in RFC3339 UTC notation.                   |
| kind        | string | The kind of entrants this tournament is for: either `"player"` or `"team"`. |

## GET `/v3/tournaments`

Returns a list of all tournaments.

### Response Body

Returns a list of partial tournaments with the following fields:

| Field       | Type   | Description                                                                 |
| ----------- | ------ | --------------------------------------------------------------------------- |
| id          | u64    | A unique identifier for the tournament.                                     |
| name        | string | The name of the tournament.                                                 |
| date        | string | Datetime of the tournament start in RFC3339 UTC notation.                   |
| kind        | string | The kind of entrants this tournament is for: either `"player"` or `"team"`. |

## GET `/v3/tournaments/:id`

Returns the [Tournament](#tournament) with the given `ìd`.

### Response Body

Returns the complete [Tournament](#tournament).

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## POST `/v3/tournaments`

Creates a new tournament from the request body. Returns the newly created ressource on success.

### Request Headers

Requires the following headers to be set:
- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains the following fields:

| Field       | Type   | Description                                                                 |
| ----------- | ------ | --------------------------------------------------------------------------- |
| name        | string | The name of the tournament.                                                 |
| description | string | The description of the tournament in an markdown format.                    |
| date        | string | Datetime of the tournament start in RFC3339.                                |
| kind        | string | The kind of entrants this tournament is for: either `"player"` or `"team"`. |

### Response Body

Returns the newly created [Tournament](#tournament).

### Errors

- `400 Bad Request`: Returned if the request body is malformed or fields are missing.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `403 Forbidden`: Returned if the token provided in the `Authorization` header is valid, but is lacking the required permissions.

## PATCH `/v3/tournaments/:id`

Updates a number of fields of the tournament with the given `id`. Patching some fields may require previous updates to other resources:

| Field | Note                                                                                                                                      |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| id    | Cannot be updated. If sent, this field is always ignored.                                                                                 |
| kind  | Changing this field requires the tournament having no entrants. If sent with the same value as the existing value, this field is ignored. |

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains all fields to be updated. All [Tournament](#tournament) fields are avaliable.

### Response Body

Returns the updated [Tournament](#tournament).

### Errors

- `400 Bad Request`: Returned if the request body is malformed.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `403 Forbidden`: Returned if the token provided in the `Authorization` header is valid, but is lacking the required permissions.
- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## DELETE `/v3/tournaments/:id`

Permanently deletes the tournament with the given `id`. This includes all sub-endpoints of the tournament and cannot be undone.

### Request Headers

Requires the following headers to be set:
- `Authorization: Bearer <TOKEN>`

### Errors

- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `403 Forbidden`: Returned if the token provided in the `Authorization` header is valid, but is lacking the required permissions.
- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## Sub-Endpoints

Each tournament has the following endpoints:
- /v3/tournaments/:id/brackets
- /v3/tournaments/:id/entrants
- /v3/tournaments/:id/roles
