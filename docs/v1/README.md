# V1

## **WARNING: V1 IS DEPRECIATED**

## API Endpoints

- /v1/tournament
- /v1/auth

### GET /v1/tournament

Returns a list of all tournaments with the following fields each:

`id`: *u64*, The unique id of the tournament  
`name`: *string*, The name of the tournament  
`bracket_type`: `0` for single elimination or `1` for double elimination  
`best_of`: *u64*, The number of games required to win a match  
`teams`: *u64*, The number of teams playing in the tournament

### GET /v1/tournament/:id

Returns all detailed data about the tournament with the given `id`. The following fields are returned:

`id`: *u64*, The unique id of the tournament  
`name`: *string*, The name of the tournament  
`bracket_type`: `0` for single elimination or `1` for double elimination  
`best_of`: *u64*, The number of games required to win a match  
`teams`: *[Team]*, All teams playing in the tournament

**Team fields:**  
`name`: *string*, The name of the team  
`players`: *[Player]*, All players in the team  

**Player fields:**  
`accountName`: *string*, The gw2 account name of the player  
`role`: `0` for *Unknown*, `1` for *Roamer*, `2` for *Teamfighter*, `3` for *Duelist*, `4` for *Support*  

#### Errors

`404`: If the tournament with the given `id` does not exist.

### POST /v1/tournament

Creates a new tournament using the data in the request body. Returns the `id` of the newly created tournament on success.

#### Request Body

The request body contains the following fields:  
`name`: *string*, The name of the tournament  
`bracket_type`: `0` for single elimination, `1` for double elimination  
`best_of`: *u64*, The number of games required to win a match  
`teams`: *[Team]*, All teams playing in the tournament

**Team fields:**  
`name`: *string*, The name of the team  
`players`: *[Player]*, All players in the team  

**Player fields:**  
`accountName`: *string*, The gw2 account name of the player  
`role`: `0` for *Unknown*, `1` for *Roamer*, `2` for *Teamfighter*, `3` for *Duelist*, `4` for *Support*  

#### Request Headers

`Content-Type`: `application/json`  
`Authorization`: `Basic <USERNAME:PASSWORD>`

#### Errors

`400`: Returned if the request body is malformed or is missing required fields.  
`401`: Returned if the credentials provided in the `Authorization` header are invalid.

### GET /v1/tournament/:id/bracket

Returns the current state of the bracket for the tournament with the given `id`. Returns the list of matches in the bracket. Each match contains the following fields:

`entrants`: *[EntrantSpot]*, The entrants in the match. Always contains exactly two elements.

**EntrantSpot:**  
An *EntrantSpot* can either be a keyless *string* or an *string* key with an object:  
`TBD`: *string*  
`Empty`: *string*  
`Entrant`: *Team*

**Team:**  
`name`: *string*, The name of the team  
`players`: *[Player]*, The players in the team  
`score`: *u64*, The score of the team in the current match  
`winner`: *bool*, Whether the team has won the current match

**Player:**  
`accountName`: *string*, The gw2 account name of the player  
`role`: `0` for *Unknown*, `1` for *Roamer*, `2` for *Teamfighter*, `3` for *Duelist*, `4` for *Support*  

#### Errors

`404`: Returned if the tournament with the given `id` does not exist or no bracket for the tournament exists.  

### PUT /v1/tournament/:id/bracket

Updates the state of the bracket for the tournament with the given `id`.

#### Request Body

Takes the exact body format that is returned when *GET*-ing the same endpoint (see above).

#### Request Headers

`Content-Type`: `application/json`  
`Authorization`: `Basic <USERNAME:PASSWORD>`

#### Errors

`400`: Returned if the request body is malformed or is missing required fields.  
`401`: Returned if the credentials provided in the `Authorization` header are invalid.  
`404`: Returned if the tournament with the given `id` does not exist.

### POST /v1/auth/login

Log in using the given credentials.

**Note**: This endpoint does not return any data response body. If the credentails are correct you can use them by adding `Authorization`: `Basic <USERNAME:PASSWORD>` to request headers.

#### Request Body

`username`: *string*, The username to authorize  
`password`: *string*, The password for the username

#### Request Headers

`Content-Type`: `application/json`

#### Errors

`401`: Returned if the provided credentials are invalid.
