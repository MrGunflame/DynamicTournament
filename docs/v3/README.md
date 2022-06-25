# V3

## Api Endpoints

- [/v3/auth](auth.md)
- [/v3/systems](systems.md)
- [/v3/tournaments](tournaments.md)

## Authentication

Authentication for write-endpoints is provided via a [jwt.io](JWT) the `Authorization` header. If the header is missing for a request a `401` error
is returned.
