# Greq Complete Documentation

This document provides comprehensive information about all Greq features, syntax, and capabilities.

## Table of Contents

1. [File Structure](#file-structure)
2. [Header Properties](#header-properties)
3. [Content Section](#content-section)
4. [Footer Conditions](#footer-conditions)
5. [Inheritance System](#inheritance-system)
6. [Dependencies](#dependencies)
7. [Placeholders](#placeholders)
8. [Command Line Interface](#command-line-interface)
9. [Examples](#examples)

## Greq file structure

Greq files (`.greq`) consist of 3 sections separated by delimiter lines:

```greq
[Header Section]
property: value

====

[Content Section]
HTTP Request

====

[Footer Section]
condition: value
```

### Section Delimiters

- **Default**: At least 4 similar, non alpha-numeric characters (`====`) on one line
- **Custom**: Set via `delimiter` property in the header part (e.g., `delimiter: $`)
- Empty lines and lines starting with `--` are ignored in the header and the footer sections

## Header Properties

| Property | Description | Example | Default |
|----------|-------------|---------|---------|
| `project` | Test project name | `project: User API Tests` | None |
| `is-http` | Use HTTP instead of HTTPS | `is-http: true` | `false` |
| `delimiter` | Section separator character | `delimiter: $` | `=` |
| `extends` | Base file to inherit from | `extends: base-config.greq` | None |
| `depends-on` | File to execute first | `depends-on: auth-setup.greq` | None |
| `timeout` | Request timeout in milliseconds | `timeout: 5000` | `30000` |
| `number-of-retries` | Retry attempts on failure | `number-of-retries: 3` | `0` |

### Property Details

#### `extends`
Inherits configuration from another `.greq` file. The base file's header and content are merged with the current file, with the current file taking precedence.

#### `depends-on`
Executes another test file before this one and makes its response available for placeholder replacement. Can reference files with or without `.greq` extension.

#### `timeout`
Maximum time to wait for a response in milliseconds. Requests exceeding this time will fail.

## Content Section

The content section contains a raw HTTP request following RFC 7230 format.

### Request Line
```http
METHOD /path/to/resource HTTP/1.1
```

**Supported Methods**: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS

### Headers
```http
host: api.example.com
content-type: application/json
authorization: Bearer token123
```

**Required**: `host` header (can be inherited from base file)

### Request Body
Separated from headers by an empty line:

```http
POST /users HTTP/1.1
host: api.example.com
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

## Footer Conditions

The footer section defines conditions to validate the HTTP response.

### Condition Syntax
```
[or] [not] [case-sensitive] property operator: value
```

### Available Properties

| Property | Description | Example |
|----------|-------------|---------|
| `status-code` | HTTP status code | `status-code equals: 200` |
| `latency` | Response time in milliseconds | `latency less-than: 1000` |
| `headers` | All response headers | `headers contains: content-type` |
| `headers.name` | Specific header | `headers.content-type contains: json` |
| `response-body` | Response body content | `response-body contains: success` |
| `response-body.path` | JSON path in response | `response-body.json.user.id equals: 123` |

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `equals` | Exact match | `status-code equals: 201` |
| `contains` | String contains | `response-body contains: error` |
| `matches-regex` | Regular expression match | `response-body matches-regex: ^[A-Z]+$` |
| `less-than` | Numeric comparison | `latency less-than: 5000` |
| `less-than-or-equal` | Numeric comparison | `status-code less-than-or-equal: 299` |
| `greater-than` | Numeric comparison | `latency greater-than: 100` |
| `greater-than-or-equal` | Numeric comparison | `status-code greater-than-or-equal: 200` |
| `exists` | Numeric comparison | `status-code greater-than-or-equal: 200` |

### Condition Modifiers

#### `or`
Combines conditions with logical OR (default is AND):
```greq
status-code equals: 200
or status-code equals: 201
```

#### `not`
Negates the condition:
```greq
not response-body contains: error
```

#### `case-sensitive`
Makes string comparisons case-sensitive:
```greq
case-sensitive response-body contains: SUCCESS
```

### JSON Path Navigation

Navigate JSON responses using dot notation and array indices:

```greq
response-body.json.users[0].name equals: John Doe
response-body.json.metadata.version equals: 1.2.3
response-body.json.data.items[1].active equals: true
```

## Inheritance System

Use `extends` to inherit from base configuration files:

**base-config.greq:**
```greq
project: Base API Config
is-http: true

====

GET /api/health HTTP/1.1
host: api.example.com
user-agent: Greq/1.0

====

status-code less-than: 500
```

**user-test.greq:**
```greq
extends: base-config.greq
project: User API Test

====

GET /api/users HTTP/1.1

====

response-body contains: users
```

The resulting merged configuration will:
- Use `is-http: true` from base
- Override project name to "User API Test"
- Use `host` and `user-agent` headers from base
- Override the path to `/api/users`
- Combine footer conditions

## Dependencies

Use `depends-on` to chain tests and extract values from previous responses:

**auth.greq:**
```greq
project: Authentication
is-http: true

====

POST /auth/login HTTP/1.1
host: api.example.com
content-type: application/json

{
  "username": "admin",
  "password": "secret"
}

====

status-code equals: 200
response-body.json.token exists: true
```

**protected-resource.greq:**
```greq
project: Protected Resource
depends-on: auth.greq
is-http: true

====

GET /api/protected HTTP/1.1
host: api.example.com
authorization: Bearer $(dependency.response-body.token)

====

status-code equals: 200
```

## Placeholders

Extract and reuse values from dependency responses using placeholder syntax:

### Placeholder Format
```
$(dependency.property-name)
```

### Available Properties

| Property | Description | Example |
|----------|-------------|---------|
| `status-code` | HTTP status code | `$(dependency.status-code)` |
| `latency` | Response time in ms | `$(dependency.latency)` |
| `headers.name` | Response header | `$(dependency.headers.set-cookie)` |
| `response-body.<path>` | JSON path | `$(dependency.response-body.user.id)` |

### Placeholder Examples

```greq
# Use status code
GET /status/$(dependency.status-code) HTTP/1.1

# Use response header
authorization: $(dependency.headers.authorization)

# Use JSON values
GET /users/$(dependency.response-body.json.id) HTTP/1.1

# In request body
{
  "user_id": "$(dependency.response-body.json.user.id)",
  "session": "$(dependency.response-body.json.session_token)"
}
```

## Command Line Interface

### Basic Usage
```bash
cargo run -- [OPTIONS] <file1.greq> [file2.greq] [...]
```

### Options

| Option | Description |
|--------|-------------|
| `--verbose` | Enable detailed logging output |
| `--help` | Show help information |

### Examples

```bash
# Run single test
cargo run -- api-test.greq

# Run multiple tests in parallel
cargo run -- auth.greq users.greq posts.greq

# Run with verbose logging
cargo run -- --verbose complex-workflow.greq

# Run all tests in a directory
cargo run -- tests/*.greq
```

### Output Format

Greq provides clear, colored output showing:
- Test execution progress
- Response status and timing
- Condition evaluation results
- Summary of passed/failed tests

Example output:
```
Greq - Web API Tester
==============================
✓ auth.greq - All conditions passed
✓ users.greq - All conditions passed
✗ posts.greq - 1 condition(s) failed
  Failed: status-code equals '201'

=== Execution Results ===
✓ auth.greq
  Status: 200 (245ms)

✓ users.greq
  Status: 200 (156ms)

✗ posts.greq
  Status: 400 (89ms)

Summary: 2 passed, 1 failed
```

## Examples

### Basic GET Request
```greq
project: Simple API Test
is-http: true

====

GET /api/health HTTP/1.1
host: httpbin.org

====

status-code equals: 200
response-body contains: ok
```

### POST with JSON
```greq
project: Create User
is-http: true

====

POST /post HTTP/1.1
host: httpbin.org
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

====

status-code equals: 200
response-body.json.json.name equals: John Doe
```

### Complex Conditions
```greq
project: Complex Validation
is-http: true

====

PUT /put HTTP/1.1
host: httpbin.org
content-type: application/json

{
  "operation": "update",
  "data": {"id": 123}
}

====

status-code equals: 200
or status-code equals: 201
response-body contains: json
not response-body contains: error
latency less-than: 5000
headers contains: content-type
```

### Inheritance Example
```greq
extends: base-api.greq
project: User Management

====

GET /users/123 HTTP/1.1

====

response-body.json.id equals: 123
response-body.json.name exists: true
```

### Dependency Chain
```greq
depends-on: create-session.greq
project: Authenticated Request

====

GET /profile HTTP/1.1
host: api.example.com
authorization: Bearer $(dependency.response-body.token)

====

status-code equals: 200
response-body.json.username exists: true
```

This documentation covers all major features and syntax of the Greq testing tool. For additional examples, see the `greq-examples/` directory in the project repository.
