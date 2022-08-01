# `/v2/tournament`

The `/v2/tournament` endpoint exposes the avaliable tournaments. Note that unlike the name suggests, this endpoint returns a list of tournaments.

### GET `/v2/tournament`

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

### GET `/v2/tournament/:id`

Returns the tournament with the requested `id`.

### Response Body

| Field        | Type   | Description |
| ------------ | ------ | --- |
| id           | u64    | The unqiue id of the tournament |
| name         | string | The name of the tournament |
| description  | string | A more detailed description of the tournament |
| date         | string | The starting datetime of the tournament in RFC3339 UTC format |
| bracket_type | string | The type of the bracket. Either `"single_elimination"` or `"double_elimination"` for single elimination or double elimination brackets respectively |
| entrants     | object | An object containing a key `players` with a []Player value of a key `teams` with a []Team value. |

### Errors

- `404 Not Found`: Returned if the tournament with the requested `id` does not exist.
