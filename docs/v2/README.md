# V2

## API Endpoints

- [/v2/auth](auth.md)
- [/v2/tournament](tournament.md)

## Authentication

Authentication for write-endpoints is provided via a [JWT](https://jwt.io) in the `Authorization` header. If the header is missing for a request, a `401` error status is returned.
