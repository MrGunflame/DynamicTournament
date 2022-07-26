# /v3/systems

The `/v3/systems` endpoint provides all bracket types avaliable for tournaments.

## Types

### System

| Field   | Type   | Description                                                                                           |
| ------- | ------ | ----------------------------------------------------------------------------------------------------- |
| id      | u64    | A unique identifier for a system. This is referenced in other parts of the api.                        |
| name    | string | The name of the system.                                                                               |
| options | object | A list of optional settings for the system. The value of the object is a [OptionValue](#optionvalue). |

### OptionValue

| Field | Type   | Description                              |
| ----- | ------ | ---------------------------------------- |
| name  | string | The full name/description of the option. |
| value | bool   | i64 | u64 | string | The default value of the option. The value can be a bool, i64, u64 or string. If providing the value it must be of the same type as the default value. |


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
