# /v3/tournaments/:id/entrants

This sub-endpoint is avaliable for all tournaments. It is used to create, delete or update the entrants of the tournament.

## Types

An entrant can be one of two:

### Player

| Field  | Type            | Description                                                                  |
| ------ | --------------- | ---------------------------------------------------------------------------- |
| name   | string          | The name of the player.                                                      |
| rating | u64 &#124; null | The rating of the player or `null` to disable. Optional, defaults to `null`. |

### Team

| Field   | Type     | Description                        |
| ------- | -------- | ---------------------------------- |
| name    | string   | The name of the team.              |
| players | Player[] | A list of all players in the team. |

