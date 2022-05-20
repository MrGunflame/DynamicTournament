# V2

## API Endpoints

- /v2/tournament
- /v2/auth

## Authentication

Authentication is provided via the `Authorization` header. v2 uses JWT tokens for authentication, basic auth is not supported anymore. For more information read the section about `/v1/auth`.

### GET /v1/tournament

Returns a list of tournaments with the following fields each:

`ìd`: *u64*, The unique id of the tournament  
`name`: *string*, The name of the tournament  
`date`: *string*, The starting datetime of the tournament in RFC3339 with UTC format  
`bracket_type`: `single_elimination` or `double_elimination` for single elimination or double elimination tournaments respectively  
`entrants`: *u64*, The number of entrants playing in the tournament

### GET /v1/tournament/:id

Returns all detailed information about the tournament with the given `id`. The following fields are returned:

`id`: *u64*, The unique id of the tournament  
`name`: *string*, The name of the tournament  
`description`: *string*, A more detailed description about the tournament  
`date`: *string*, The starting datetime of the tournament in RFC3339 with UTC format  
`bracket_type`: `single_elimination` or `double_elimination` for single elimination or double elimination tournaments respectively  
`entrants`: *object*, An object containing a key `players` with a *[Player]* value **or** a key `teams` with a *[Team]* value.

Player fields:  
`name`: *string*, The name of the player
`role`: `unknown` for *Unknown*, `roamer` for *Roamer*, `teamfighter` for *Teamfighter*, `duelist` for *Duelist*, `support` for *Support*  
`rating`: *u64 | null*, The rating of the player.

Team fields:  
`name`: *string*, The name of the team  
`players`: *[Player]*, All players of the team

#### Errors

`404`: Returned if the tournament with the given `id` does not exist.

### POST /v1/tournament

Creates a new tournament using the data in the request body. Returns the `id` of the newly created tournament on success.

#### Request Body

The request body contains the following fields:  
`name`: *string*, The name of the tournament  
`description`: *string*, A more detailed description about the tournament  
`date`: *string*, The starting datetime of the tournament in RFC3339 (UTC format not necessary)  
`bracket_type`: `single_elimination` or `double_elimination` for single elimination or double elimination tournaments respectively  
`entrants`: *object*, An object containing a key `players` with a *[Player]* value **or** a key `teams` with a *[Team]* value.

Player fields:  
`name`: *string*, The name of the player  
`role`: `unknown` for *Unknown*, `roamer` for *Roamer*, `teamfighter` for *Teamfighter*, `duelist` for *Duelist*, `support` for *Support*  
`rating`: (optional) *u64 | null*, The rating of the player. If the player is in a team the rating is used to calculate the team rating. The default value if omitted is `null`.


Team fields:  
`name`: *string*, The name of the team  
`players`: *[Player]*, All players of the team

#### Request Headers

`Content-Type`: `application/json`  
`Content-Length`: `<BODY_BYTES_LENGTH>`  
`Authorization`: `Bearer <AUTH_TOKEN>`

#### Errors

`400`: Returned if the request body is malformed or is missing required fields.  
`401`: Returned if the credentials provided via the `Authorization` header are invalid.
