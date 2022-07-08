# /v3/systems

The `/v3/systems` endpoint provides all bracket types avaliable for tournaments.

## Types

### System

| Field | Type   | Description                                                                     |
| ----- | ------ | ------------------------------------------------------------------------------- |
| id    | u64    | A unique identifier for a system. This is referenced in other parts of the api. |
| name  | string | The name of the system.                                                         |

## GET `/v3/systems`

Returns a list of all [System](#system) avaliable.

### Response Body

Returns a list of partial [Systems](#system) containing only the following fields:

| Field | Type   | Description                                                                     |
| ----- | ------ | ------------------------------------------------------------------------------- |
| id    | u64    | A unique identifier for a system. This is referenced in other parts of the api. |
| name  | string | The name of the system.                                                         |

## GET `/v3/systems/:id`

Returns a single complete [System](#system) with the given `id`.

### Reponse Body

Returns a single complete [System](#system).

### Errors

`404 Not Found`: Returned if the system with the given `id` does not exist.
