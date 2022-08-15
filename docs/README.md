# API Docs

## API Versions

| Version | Stable  | Depreciated      | Documentation |
| ------- | ------- | ---------------- | ------------- |
| v3      | Partial | No               | [Link](v3)   |
| v2      | Yes     | No               | [Link](v2)   |
| v1      | Yes     | Yes (2022-06-06) | [Link](v1)   |

## Stability

When an API version is considered *stable*, it is guaranteed to stay backwards-compatible. This means:
- no fields are removed or renamed
- no types of fields are changed
- the meaning or value of the field is not changed in a way that breaks the existing meaning/value

A stable API does not guarantee no changes to the API. The following changes are possible under a stable API:
- new fields may be added
- the meaning or value of a field is changed in an additive way

An API that is considered *partially stable* is only unstable for specific endpoints when documented so. Completely undocumented sections are also considered unstable.
