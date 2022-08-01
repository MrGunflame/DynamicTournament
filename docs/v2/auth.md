# /v2/auth

The `/v2/auth` endpoint provides authentication endpoints. Using authentication is only required for read-endpoints.

If possible you should use `/v3/auth` instead. It provides the exact same api as `/v2/auth`, but comes from the newer version. You can use tokens from `/v3/auth` for v2 endpoints.

## POST `/v2/auth/login`

Log in using the given credentials and acquire a new auth and refresh token.

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

- `401 Unauthorized`: Returned if the credentials in the request body were invalid.

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

- `401 Unauthorized`: Returned if the refresh token in the request body was invalid (e.g. expired).
