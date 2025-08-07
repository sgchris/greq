# Greq 🚀

A robust web API testing tool with inheritance, dependencies, and dynamic request support built in Rust.

## Overview

Greq processes test definitions from `.greq` files, enabling you to create sophisticated HTTP API test suites with features like:

- **Inheritance**: Extend base configurations across multiple test files
- **Dependencies**: Execute tests in sequence with data flow between them
- **Dynamic Placeholders**: Extract and reuse values from responses
- **Parallel Execution**: Run multiple independent tests simultaneously
- **Rich Conditions**: Comprehensive response validation with multiple operators

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
response-body.json.id exists: true
response-body.json.name equals: John Doe
```

## Features

- 🔗 **File Inheritance**: Share common configurations with `extends`
- 📦 **Dependencies**: Chain tests together with `depends-on`
- 🔄 **Dynamic Values**: Extract and reuse response data with placeholders
- ⚡ **Parallel Execution**: Automatic parallel processing of independent tests
- 📊 **Rich Validation**: Status codes, headers, JSON paths, response times
- 📝 **Detailed Logging**: Comprehensive logging with configurable verbosity
- 🎯 **Zero Configuration**: Works out of the box with sensible defaults

## Documentation

For complete documentation including all properties, operators, and advanced features, see:

📖 **[Complete Documentation](docs/documentation.md)**

## Project Structure

```
greq/
├── src/           # Core application code
├── greq-examples/ # Example test files
├── docs/          # Comprehensive documentation
└── target/        # Build artifacts
```

## Examples

The `greq-examples/` directory contains various example files demonstrating different features:

- **Basic tests**: Simple GET/POST requests
- **Inheritance**: Using base configurations
- **Dependencies**: Chaining tests with data flow
- **Advanced conditions**: Complex validation scenarios

## Contributing

This project follows idiomatic Rust patterns and coding standards. Please ensure all code:

- Compiles without warnings (`cargo clippy`)
- Is properly formatted (`cargo fmt`)
- Includes appropriate tests (`cargo test`)
- Follows the project's error handling patterns

## License

This project is open source. See LICENSE file for details.
