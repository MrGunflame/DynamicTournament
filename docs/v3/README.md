# V3

## Api Endpoints

- [/v3/auth](auth.md)
- [/v3/systems](systems.md)
- [/v3/tournaments](tournaments.md)

## Authentication

Authentication for write-endpoints is provided via a [JWT](https://jwt.io) in the `Authorization` header. If the header is missing for a request a `401` error
is returned.

## Request Headers

For read-only requests no request headers are required. For write requests additional headers may be required:

- `Authorization`: This header is always required when requesting write operations.
- `Content-Length`: This header is always required for requests with a request body. If not present the request will be rejected.
- `Content-Type`: This header is not required, but recommended for requests with a request body. The server will always assume the request body is `application/json`.
