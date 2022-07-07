# /v3/auth

`/v3/auth` provides the endpoints used for acquiring and refreshing JWTs.

## POST `/v3/auth/login`

Login in using the given credentials and acquire a new auth and refresh token.

### Request Body

| Field    | Type   | Description          |
| -------- | ------ | -------------------- |
| username | string | Username of the user |
| password | string | Password of the user |

### Response Body

| Field         | Type   | Description                                          |
| ------------- | ------ | ---------------------------------------------------- |
| auth_token    | string | The authentication token (use this to make requests) |
| refresh_token | string | The refresh token (use this for `/v3/auth/refresh`)  |

### Errors

`401 Unauthorized`: Returned if the credentials in the request body were invalid.

## POST `/v3/auth/refresh`

Acquire a new authentication and refresh token using an existing valid refresh token.

### Request Body

| Field         | Type   | Description           |
| ------------- | ------ | --------------------- |
| refresh_token | string | A valid refresh token |

### Response Body

| Field         | Type   | Description                                          |
| ------------- | ------ | ---------------------------------------------------- |
| auth_token    | string | The authentication token (use this to make requests) |
| refresh_token | string | The refresh token (use this for `/v3/auth/refresh`)  |

### Errors

`401 Unauthorized`: Returned if the refresh token in the request body was invalid (e.g. expired).
