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
| `allow-dependency-failure` | Continue if dependency fails | `allow-dependency-failure: false` | `true` |
| `show-warnings` | Show warning messages during execution | `show-warnings: false` | `true` |
| `timeout` | Request timeout in milliseconds | `timeout: 5000` | `30000` |
| `number-of-retries` | Retry attempts on failure | `number-of-retries: 3` | `0` |
| `execute-before` | Shell command to run before HTTP request | `execute-before: echo "Starting test"` | None |
| `execute-after` | Shell command to run after response received | `execute-after: echo "Test completed"` | None |
| `set-environment.<name>` | Set environment variable for subsequent requests | `set-environment.AUTH_TOKEN: $(dependency.response-body.token)` | None |

### Property Details

#### `extends`
Inherits configuration from another `.greq` file. The base file's header and content are merged with the current file, with the current file taking precedence.

#### `depends-on`
Executes another test file before this one and makes its response available for placeholder replacement. Can reference files with or without `.greq` extension.

#### `allow-dependency-failure`
When set to `false`, stops execution if the dependency defined by `depends-on` fails. By default (`true`), allows the current test to continue executing even if dependencies fail. Useful for robust test workflows where dependencies might legitimately fail.

#### `show-warnings`
Controls whether warning messages are displayed during execution. When set to `false`, suppresses warnings such as placeholder replacement notifications when dependencies fail. Default is `true`.

#### `timeout`
Maximum time to wait for a response in milliseconds. Requests exceeding this time will fail.

#### `execute-before`
Executes a shell command before sending the HTTP request. The command runs after dependency resolution and placeholder replacement, but before the actual HTTP request is sent. This is useful for:
- Setting up test data
- Preparing test environment
- Logging test start
- Executing setup scripts

The command supports placeholders for both environment variables and dependency responses (if `depends-on` is specified).

**Example:**
```greq
execute-before: echo "Preparing test environment"
execute-before: ./scripts/setup-test-data.sh
execute-before: echo "API Key: $(environment.API_KEY)"
```

**Important:** If the execute-before command fails (non-zero exit code), the test is marked as failed and execution stops.

#### `execute-after`
Executes a shell command after receiving the HTTP response and evaluating all conditions. The command runs regardless of whether conditions passed or failed. This is useful for:
- Cleaning up test data
- Logging test results
- Sending notifications
- Executing teardown scripts

The command supports placeholders for environment variables, dependency responses, and the current response.

**Example:**
```greq
execute-after: echo "Test completed with status: $(dependency.status-code)"
execute-after: ./scripts/cleanup.sh
execute-after: echo "Response body: $(dependency.response-body)"
```

**Important:** If the execute-after command fails, the test is marked as failed even if all conditions passed.

#### `set-environment.<variable_name>`
Sets environment variables that can be used in subsequent requests within the same execution session. The variable name is specified after the dot, and the value supports full placeholder replacement including dependency responses.

This is particularly useful for:
- Capturing authentication tokens from login responses
- Storing user IDs, session tokens, or API keys
- Passing data between test files
- Setting dynamic configuration values

**Key Features:**
- Variables are set **after** placeholder replacement, so dependency values are fully resolved
- Multiple variables can be set in a single file
- Variables persist for the entire execution session and are available to all subsequent requests
- Works seamlessly with `depends-on` to capture response data
- Can be used in combination with `execute-before` and `execute-after` commands

**Example - Capturing Auth Token:**
```greq
depends-on: login.greq
set-environment.GREQ_AUTH_TOKEN: $(dependency.response-body.auth_token)
set-environment.GREQ_USER_ID: $(dependency.response-body.user.id)

====

GET /api/protected HTTP/1.1
host: api.example.com
authorization: Bearer $(environment.GREQ_AUTH_TOKEN)
x-user-id: $(environment.GREQ_USER_ID)
```

**Example - Static Configuration:**
```greq
set-environment.API_VERSION: v2
set-environment.TENANT_ID: tenant-123
set-environment.API_KEY: my-secret-key

====

GET /api/$(environment.API_VERSION)/users HTTP/1.1
host: api.example.com
x-api-key: $(environment.API_KEY)
x-tenant-id: $(environment.TENANT_ID)
```

**Important Notes:**
- Variable names are case-insensitive when parsed (converted to lowercase)
- Environment variables set in one greq file are available to all subsequent files in the execution chain
- Use the `$(environment.<variable_name>)` syntax to access the variables in requests
- Variables are set in the order they appear in the file

#### Shell Command Execution Details

**Windows**: Commands are executed using `powershell.exe -Command`
**Unix/Linux/Mac**: Commands are executed using `sh -c`

**Working Directory**: Commands run in the same directory as the greq file.

**Placeholders**: Both execute-before and execute-after support full placeholder replacement:
- `$(environment.VAR_NAME)` - Environment variables
- `$(dependency.field)` or `$(dep.field)` - Data from dependency responses
- In execute-after: `$(dependency.field)` references the current response

**Example with Dependencies:**
```greq
project: User Cleanup Test
depends-on: create-user.greq
execute-before: echo "Cleaning up user ID: $(dependency.response-body.id)"
execute-after: echo "Cleanup completed with status $(dependency.status-code)"
====
DELETE /users/$(dependency.response-body.id)
host: api.example.com
====
status-code equals: 200
```

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
| `response-body.path` | JSON path in response | `response-body.user.id equals: 123` |

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
response-body.users[0].name equals: John Doe
response-body.metadata.version equals: 1.2.3
response-body.data.items[1].active equals: true
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
response-body.token exists: true
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

## Dependency Failure Handling

By default, dependency failures are allowed and execution continues (`allow-dependency-failure: true`). If you want execution to stop when a dependency fails, you can explicitly set `allow-dependency-failure: false`.

### Use Cases

Common scenarios where you might want to allow dependency failures:

1. **Cleanup operations**: Deleting resources that may or may not exist
2. **Optional setup**: Setup steps that aren't critical for the main test
3. **Retry scenarios**: Tests that attempt to clean up before creating

### Example: Delete Before Create

**cleanup.greq:**
```greq
project: Delete User (May Fail)
is-http: true

====

DELETE /users/123 HTTP/1.1
host: api.example.com
authorization: Bearer $(environment.api-token)

====

status-code equals: 204
```

**create-user.greq:**
```greq
project: Create User After Cleanup
depends-on: cleanup.greq
allow-dependency-failure: true
is-http: true

====

POST /users HTTP/1.1
host: api.example.com
authorization: Bearer $(environment.api-token)
content-type: application/json

{
  "id": 123,
  "name": "John Doe"
}

====

status-code equals: 201
```

In this example, if the DELETE fails (user doesn't exist), the CREATE will still proceed. The console will show:

```
⚠ Dependency 'cleanup.greq' failed but continuing (allow-dependency-failure enabled)
✓ create-user.greq Status: 201 (123ms)
```

### Dependency Failure and Placeholders

When a dependency fails and `allow-dependency-failure: true` is set, any dependency placeholders in the current file will be handled as follows:

1. **Dependency placeholders** (`$(dependency.*)`) are replaced with empty strings
2. **Environment placeholders** (`$(environment.*)`) continue to work normally
3. **Warning message** is shown for the first placeholder found (if `show-warnings: true`)

#### Example with Placeholders

**auth-setup.greq:**
```greq
project: Authentication Setup
is-http: true

====

POST /auth/login HTTP/1.1
host: api.example.com
content-type: application/json

{"username": "test", "password": "secret"}

====

status-code equals: 200
response-body contains: token
```

**main-test.greq:**
```greq
project: Main Test with Optional Auth
depends-on: auth-setup
allow-dependency-failure: true
show-warnings: true
is-http: true

====

GET /protected/resource HTTP/1.1
host: api.example.com
authorization: Bearer $(dependency.response-body.token)
x-fallback-auth: $(environment.FALLBACK_TOKEN)

====

status-code equals: 200
```

If `auth-setup.greq` fails, the console will show:

```
⚠ Dependency 'auth-setup.greq' failed but continuing (allow-dependency-failure enabled)
⚠ Warning: main-test.greq: header 'authorization': Dependency placeholder found but dependency failed. Placeholder will be replaced with empty string.
✓ main-test.greq Status: 200 (156ms)
```

The request will be sent with:
- `authorization: Bearer ` (empty token, just "Bearer ")
- `x-fallback-auth: your-fallback-token` (environment variable still works)

#### Controlling Warning Messages

Use `show-warnings: false` to suppress placeholder warning messages:

```greq
project: Silent Dependency Failure
depends-on: auth-setup
allow-dependency-failure: true
show-warnings: false
```

This will still replace dependency placeholders with empty strings but won't show warnings.

## Placeholders

Extract and reuse values from dependency responses and environment variables using placeholder syntax:

### Placeholder Formats
```
$(dependency.property-name)      # Dependency response values
$(environment.variable-name)     # Environment variables
```

### Dependency Properties

| Property | Description | Example |
|----------|-------------|---------|
| `status-code` | HTTP status code | `$(dependency.status-code)` |
| `latency` | Response time in ms | `$(dependency.latency)` |
| `headers.name` | Response header | `$(dependency.headers.set-cookie)` |
| `response-body.<path>` | JSON path | `$(dependency.response-body.user.id)` |

### Environment Variables

Access environment variables using the `$(environment.variable-name)` syntax:

```greq
# Use environment variables
authorization: Bearer $(environment.api-token)
host: $(environment.api-host)

# In request body
{
  "api_key": "$(environment.api-key)",
  "version": "$(environment.app-version)"
}
```

### Placeholder Examples

```greq
# Use status code
GET /status/$(dependency.status-code) HTTP/1.1

# Use environment variables
authorization: Bearer $(environment.api-token)

# Use response header
authorization: $(dependency.headers.authorization)

# Use JSON values
GET /users/$(dependency.response-body.id) HTTP/1.1

# In request body
{
  "user_id": "$(dependency.response-body.user.id)",
  "session": "$(dependency.response-body.session_token)"
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
response-body.name equals: John Doe
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

### Environment Variables Examples

**Development Environment Test:**
```greq
project: Environment API Test
is-http: true

====

GET /api/v1/users HTTP/1.1
host: $(environment.api-host)
authorization: Bearer $(environment.api-token)
x-client-version: $(environment.app-version)

====

status-code equals: 200
response-body.status equals: success
```

**POST with Environment Variables:**
```greq
project: Create Resource
is-http: true

====

POST /api/resources HTTP/1.1
host: $(environment.api-host)
authorization: Bearer $(environment.api-token)
content-type: application/json

{
  "name": "test-resource",
  "environment": "$(environment.deploy-env)",
  "api_key": "$(environment.service-key)"
}

====

status-code equals: 201
response-body.id exists: true
```

To run these tests, set the required environment variables:
```powershell
$env:API_HOST = "api.example.com"
$env:API_TOKEN = "your-token-here"
$env:APP_VERSION = "1.0.0"
$env:DEPLOY_ENV = "development"
$env:SERVICE_KEY = "service-secret"
```

### Inheritance Example
```greq
extends: base-api.greq
project: User Management

====

GET /users/123 HTTP/1.1

====

response-body.id equals: 123
response-body.name exists: true
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
====

response-body.username exists: true
```

This documentation covers all major features and syntax of the Greq testing tool. For additional examples, see the `greq-examples/` directory in the project repository.
