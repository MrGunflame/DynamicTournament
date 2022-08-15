# /v3/tournaments/:id/brackets/:id/matches

## Note: The websocket protocol is currently unstable.

This sub-endpoint is avaliable for all tournaments. It is only used to create a websocket connection and send/receive live updates the bracket state.

## GET `/v3/tournaments/:id/brackets/:id/matches`

Upgrades the http connection to a websocket connection. The connection can be used to send/receive updates of the bracket with the given `id` in realtime.

### Request Headers

Requires the following request headers to be set:
- `Upgrade: websocket`

### Response Body

The request contains contains no response body. Instead the server responds with a `101 Switching Protocols` status code and uses the connection for websocket messages.

### Errors

- `404 Not Found`: Returned if the tournament or bracket with the requested `id` does not exist.
- `426 Upgrade Required`: Returned if the request is missing the `Upgrade` header.
