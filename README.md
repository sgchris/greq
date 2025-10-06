# Greq 🚀

A tester tool for APIs with templates and dependencies.

Example:
```yaml
project: test my awesome API

-- first delete the resource if it exists
depends-on: delete-resource

====

POST /path/to/resource
Host: api.example.com
Content-Type: application/json

{"name":"Awesome resource","description":"Awesome resource used for tests"}

====

-- evaluate the response
status-code equals: 200
response-body contains: id
```

## About

`greq` is a Rust-based command-line tool for parsing, validating, and executing HTTP-like requests from `.greq` files. Ideal for API developers, maintainers, test engineers and developers working with raw HTTP interactions.

## 🚀 Features

- Parse `.greq` request files with structured metadata, body, and evaluation conditions
- Run requests and validate responses
- Supports inheritance, templating, placeholders, and chain of dependencies
- **Execute shell commands before and after HTTP requests** with `execute-before` and `execute-after`
- Environment variable placeholders with `$(environment.VAR_NAME)` syntax
- Allow dependency failure handling with `allow-dependency-failure` property
- Configurable warning system with `show-warnings` property
- Graceful placeholder replacement when dependencies fail
- Works with JSON, status codes, etc.
- Built for performance and flexibility
- Process requests simultaneously

## Quick Start

### Installation

```bash
git clone https://github.com/sgchris/greq.git
cd greq
cargo build --release
```

### Basic Usage

```bash
# Run a single test
cargo run -- test.greq

# Run multiple tests in parallel
cargo run -- auth.greq users.greq posts.greq

# Enable verbose logging
cargo run -- --verbose api-tests.greq
```

### Example Test File

```greq
project: User API Test
is-http: true
timeout: 5000

====

POST /users HTTP/1.1
host: api.example.com
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

====

status-code equals: 201
response-body.id exists: true
response-body.name equals: John Doe
```

## Documentation

For complete documentation including all properties, operators, and advanced features, see:

📖 **[Complete Documentation](docs/documentation.md)**

## Examples

The `greq-examples/` directory contains various example files demonstrating different features:

- **Basic tests**: Simple GET/POST requests
- **Inheritance**: Using base configurations
- **Dependencies**: Chaining tests with data flow
- **Dependency failure handling**: Using `allow-dependency-failure` for robust test workflows
- **Environment variables**: Using environment variable placeholders
- **Advanced conditions**: Complex validation scenarios

## Contributing

This project follows idiomatic Rust patterns and coding standards. Please ensure all code:
Please check [CONTRIBUTING](CONTRIBUTING.md) instructions.

- Compiles without warnings (`cargo clippy`)
- Is properly formatted (`cargo fmt`)
- Includes appropriate tests (`cargo test`)
- Follows the project's error handling patterns

## License

See [LICENSE](LICENSE) file for details.
