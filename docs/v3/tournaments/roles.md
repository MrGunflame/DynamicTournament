# /v3/tournaments/:id/roles

This sub-endpoint is avaliable for all tournaments. It is used to create, delete or update custom roles for players.

## Types

### Role

| Field | Type   | Description                        |
| ----- | ------ | ---------------------------------- |
| id    | u64    | The unique identifier for the role. |
| name  | string | The name of the role.              |

*Note:* The role with the `id` of 0 is reserved for the name `"Unknown"`.

## GET `/v3/tournaments/:id/roles`

Returns all [Role](#role)s avaliable for players of the tournament.

### Response Body

Returns a list of [Role](#role)s.

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## GET `/v3/tournaments/:id/roles/:id`

Returns the [Role](#role) with the given `id`.

### Response Body

Returns a [Role](#role).

### Errors

- `404 Not Found`: Returned if the tournament or role with the requested `id` does not exist.

## POST `/v3/tournaments/:id/roles`

Creates a new role.

### Request Headers

- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains the new [Role](#role) with the following fields:

| Field | Type   | Description           |
| ----- | ------ | --------------------- |
| name  | string | The name of the role. |

### Response Body

Returns the newly created [Role](#roles).

### Errors

- `400 Bad Request`: Returned if the request body is malformed or is missing fields.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## PATCH `/v3/tournaments/:id/roles/:id`

Updates the role with the given `id`.

### Request Headers

- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains the field that should be updated.

*Note:* The `id` cannot be changed. If sent, it is ignored.

### Response Body

Returns the updated [Role](#role).

### Errors

- `400 Bad Request`: Returned if the request body is malformed.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or role with the requested `id` does not exist.

## DELETE `/v3/tournaments/:id/roles/:id`

Deletes the role with the given `id`.

*Note:* The `id` of the deleted role won't be reused and any players using the deleted `id` will link to `"Unknown"` instead.

### Request Headers

- `Authorization: Bearer <TOKEN>`

### Errors

- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or role with the requested `id` does not exist.

