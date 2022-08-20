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

## Websocket protocol

The upgraded websocket connection uses a custom binary protocol. The protocol differenciates
between *Requests*, which are only sent by the client  and *Response*, which are only sent
be the server. The protocol is full-duplex: both the client and server can start sending
data independently from each other.

### Encoding

The protocol has support for encoding/decoding the following types:

| Type | Size (Bytes) |
| ---- | ------------ | 
| bool | 1            |
| u8   | 1            |
| u16  | variable     |
| u32  | variable     |
| u64  | variable     |
| i8   | 1            |
| i16  | variable     |
| i32  | variable     |
| i64  | variable     |
| [T]  | variable     |
| str  | variable     |

#### Integer encoding

All integer types with the exception of `u8` and `i8` are encoded using a variable integer 
encoding. Unsigned integers are encoded using the ULEB128 encoding. All integers represented
in little-endian.

The process of encoding an ULEB128 encoded integer is as follows:
1. Break the integer into groups of 7-bit
2. Encode the groups in little-endian
3. Set the MSB on every byte except the last

The process of decoding an ULEB128 encoded integer is as follows:
1. Read bytes from the input until a bytes doesn't have the MSB set
2. Remove every 8th bit (MSB) from the bytes, creating groups of 7-bit
3. Accumulate all bytes from little-endian

Signed integers are converted into their unsigned variant using a zigzag encoding. This 
encoding stores the sign bit in the LSB.

An signed integer `n` with `k` bits can be encoded using `(n << 1) ^ (n >> k - 1)`.

##### bool

A boolean type that is either `true` or `false`. Encoded as a u8 with a 0 representing 
`false` and a 1 representing `true`. All other byte values are invalid for this type.

##### Arrays and Strings

Arrays and strings are encoded using the same format. An array first encodes the length of 
the array, or in other words, the number of elements following. The length is a `u64` using 
the varint encoding described above. After that every element is encoded.

Strings are encoded as a array of bytes (`u8`). Note that the length is **not the number of 
characters**, but the number of bytes. In the case of ASCII this is the same, but any UTF-8 
code point is valid.

### Request

A request is from the client to the server. The first byte of the request contains the id of the
command that is being requested. After that follows the body of the command, if any.

| Command     | ID | Body | Description                                                                  |
| ----------- | -- | ---- | ---------------------------------------------------------------------------- |
| Reserved    | 0  | No   | Reserved for future use. This can safely be ignored.                         |
| Authorize   | 1  | Yes  | Authorize using an auth token. This is the same token used for the http api. |
| SyncState   | 2  | No   | Request the server to return the complete, current state of the bracket.     |
| UpdateMatch | 3  | Yes  | Update the match at a specified index. This requires authentication.         |
| ResetMatch  | 4  | Yes  | Reset the match at a specified index. This requires authentication.          |

Note that there may be more commands added in the future.

#### Authorize

The `Authorize` command authenticates the active connection using the token provided. This is the same token
used by the HTTP API. It can be acquires using the `/v3/auth` endpoint. If the provided token is rejected
(i.e. it is invalid or expired) the server will respond with an `Error::Unauthorized` error. If the token is
valid there is no response.

| Name  | Type | Description                                                                   |
| ----- | ---- | ----------------------------------------------------------------------------- |
| token | str  | The token string as returned by the `/v3/auth` endpoint. **No Bearer prefix** |

#### SyncState

The `SyncState` command requests the server to return a complete, up-to-date state of the bracket. The 
server responds with a `SyncState` response. This command has no body.

#### UpdateMatch

The `UpdateMatch` command updates the match at the specified index. The body contains the index and the 
updated
data. This command requires that the active connection is authenticated. If it is not an 
`Error::Unauthorized` response
is returned. Otherwise if this command succeeds the server will return a `UpdateMatch` response with the 
same data.

| Name  | Type           | Description |
| ----- | -------------- | -- |
| index | u64            | The index of the match. |
| nodes | []EntrantScore | An array of the updated data. This currently always has the length 2
(red/blue team). |

An `EntrantScore` contains:
| Name   | Type | Description                   |
| ------ | ---- | ----------------------------- |
| score  | u64  | The score of the node.        |
| winner | bool | Whether the node is a winner. |

#### ResetMatch

The `ResetMatch` command reset the match at the specified index. The body contains the index of
match. This command requires that the active connection is authenticated. If it is not and
`Error::Unauthorized` error is returned. Otherwise if this command succeeds the server will return
a `ResetMatch` response with the same data.

| Name  | Type | Description                      |
| ----- | ---- | -------------------------------- |
| index | u64  | The index of the match to reset. |

### Response

Responses follow the format that requests do. The first byte contains the id of the event that is
being returned. After that follows the event body, if any.

| Event       | ID | Body | Description                                                           |
| ----------- | -- | ---- | --------------------------------------------------------------------- |
| Reserved    | 0  | No   | Reserved for future use. This can safely be ignored.                  |
| Error       | 1  | Yes  | An error message.                                                     |
| SyncState   | 2  | Yes  | Contains the complete state of the bracket.                           |
| UpdateMatch | 3  | Yes  | Update the match at a specified index.                                |
| ResetMatch  | 4  | Yes  | Reset the match at a specified index.                                 |

Note that there may be more events added in the future. They can be safely ignored.

#### Error

This response indicates that an error has occured. This is mostly sent in response to a request.
The body contains the id of the error that happened.

| Error        | ID | Description |
| ------------ | -- | ----------- |
| Internal     | 0  | An unrecoverable internal server error. The connection will be dropped after this error is sent. |
| Proto        | 1  | An unspecified error in the protocol. This usually happens when the client sends an invalid request. |
| Unauthorized | 2  | A sent request required authentication, but it was set. This is also returned when an invalid token is provided. |
| Lagged       | 3  | The event queue for this connection lagged behind and some events were lost. The client may want to request `SyncState`. This usually happens when the connection is very slow. |
| ProtoInvalidBool  | 128 | A specialized protocol error: an invalid bool value was decoded. |
| ProtoInvalidSeq   | 129 | A specialized protocol error: a sequence was shorter than the provided length. |
| ProtoInvalidStr   | 130 | A specialized protocol error: a string contained an invalid UTF-8 byte sequence. |
| ProtoIntOverflow  | 131 | A specialized protocol error: A varint-encoded integer was too long. Note that this may also be returned when the varint is malformed. |

#### SyncState
