# /v3/tournaments/:id/brackets

This sub-endpoint is avaliable for all tournaments. It is used to create, delete or update the brackets of the tournamet.

## Types

### Bracket

| Field    | Type         | Description                                                                                                        |
| -------- | ------------ | ------------------------------------------------------------------------------------------------------------------ |
| id       | u64          | The unique identifier of the bracket.                                                                              |
| name     | string       | The name of the bracket.                                                                                           |
| system   | u64          | The id of the system to use for the bracket. (See [Systems](../systems.md))                                        |
| options  | OptionValues | A map of optional values for the system. Optional, defaults to an empty value (See [OptionValues](#OptionValues)). |
| entrants | u64[]        | A list of all entrants in the bracket.                                                                             |

**Note:** The order of the `entrants` field may or may not matter depending on the system in use.

### OptionValues

A key-value map to provide additional optional configuration options to the system in use. Which system accepts what values can be found using the [/v3/systems](../systems.md) endpoint. If no values are provided the systems default values are used.

`OptionValues` is an object with arbitrary string keys. The value can be any of the following values:
- bool
- u64
- i64
- string

An empty object is also valid, and the default value.

## GET `/v3/tournaments/:id/brackets`

Returns a list of all brackets.

### Response Body

Returns a list of [Bracket](#bracket)s.

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## POST `/v3/tournaments/:id/brackets`

Creates a new bracket for the tournament with the given `id`.

Please note that all entrants provided in the `entrants` field must exist in the tournament with the requested `id`.

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains the new [Bracket](#bracket) with the follwing fields:

| Field    | Type         | Description                                                                                                        |
| -------- | ------------ | ------------------------------------------------------------------------------------------------------------------ |
| name     | string       | The name of the bracket.                                                                                           |
| system   | u64          | The id of the system to use for the bracket. (See [Systems](../systems.md))                                        |
| options  | OptionValues | A map of optional values for the system. Optional, defaults to an empty value (See [OptionValues](#OptionValues)). |
| entrants | u64[]        | A list of all entrants in the bracket.                                                                             |

### Response Body

Returns the newly created bracket.

### Errors

- `400 Bad Request`: Returned if the request body is malformed, missing fields or an invalid entrant was provided.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.

## PATCH `/v3/tournaments/:id/brackets/:id`

Updates a number of fields of the bracket with the given `id`.

Please note that all entrants provided in the `entrants` field must exist in the tournament with the requested `id`.

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Request Body

The request body contains all fields to be updated.

### Response Body

Returns the updated bracket.

### Errors

- `400 Bad Request`: Returned if the request body is malformed.
- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or bracket with the requested `id` does not exist.

## DELETE `/v3/tournaments/:id/brackets/:id`

Permanently deletes the bracket with the given `id`.

### Request Headers

Requires the following request headers to be set:
- `Authorization: Bearer <TOKEN>`

### Errors

- `401 Unauthorized`: Returned if the `Authorization` header is missing or contains an invalid token.
- `404 Not Found`: Returned if the tournament or bracket with the requested `id` does not exist.
