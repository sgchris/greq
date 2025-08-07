# Greq

A robust web API tester with inheritence, dependencies and dynamic requests support.

Greq processes tests defined in `.greq` files, where each file consists of 3 parts - The header, the content and the footer. 

## `GREQ` files structure

### General format

the properties of the header and the footer are defined as words joined with dash "-". e.g. "is-http", "response-body", "depends-on".

The 3 sections are divided by default with the equal character ("=") repeated at least 4 times in the beginning of a line, when this is the only character in that line.

In the header and the footer sections, empty lines or lines starting with two dashes, "--", are ignored.

The content section must have the request line and the "host" header. "Host" header can be loaded from the base request, but the request line must be present in every greq file.

### Execution / CLI

The app is a console application that receives one or more Greq files as parameters and executes them simultaniously. The Greq files may be absolute paths or paths relative to the current folder.

Execution example:
```bash
> greq my-first-test.greq my-second-test.greq
```

The exit code should reflect the evaluation and the overall processing response. Exit code 0 - success, exit code 1 on failure.

### The header section

The header part contains metadata and properties related to the execution of the current greq file. 

Here is a list of supported properties.

| Property Name | Description | Example |
|---------------|-------------|---------|
| project | The name of the test project | `project: my API test` |
| is-http | Send the request as http or https. Default false | `is-http: true` |
| delimiter | The delimiter character used as sections separator | `delimiter: $` |
| extends | Defines the "base" greq file that the current file extends | `extends: base-request.
| number-of-retries | How many retries to execute in case of failing response from the host | `number-of-retries: 3` |
| depends-on | Defines which greq file should be executed before processing the current file. The file may be provided without the `.greq` extension. It may be an absolute path or a path relative to the current file | `depends-on: auth-setup`,<br>`depends-on: /path/to/sign-in.greq` |
| timeout | Set the max timeout for the request in milliseconds | `timeout: 5000` |


### The content section

The content section has the format of the raw HTTP request [RFC 7230](https://datatracker.ietf.org/doc/html/rfc7230#page-19). 

That format includes the request line, that has the request method, the URI and the HTTP version. For example:
```http
POST /path/to HTTP/1.1
```

The headers, for example:
```http
Host: example.com
Content-Type: application/json
```

...and the request body, preceded by an empty line (defined as `\r\n` twice)
```json

{
    "data":[
        {"name":"prop1", "value":10},
        {"name":"prop2", "value":true}
    ]
}
```

### The footer section

The footer section has list of conditions that evaluate the response of the request defined in the content section.

Every condition has the following format:
```
[or] [not] <key> <operator> [case-sensitive]: <value>
```
A few examples:
```
status-code less-than: 200
or status-code greater-than: 499
response-body contains: prop1
not headers.content-type matches-regex: ^application.*$
```

#### Available prefixes

A condition line can be preceded with one of the following keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `or` | Appends the current condition to a group of conditions along with the condition above. | `or response-body contains: prop1` |
| `not` | Negative operator of the condition | `not response-body equals: {"a":1}` |


#### Available keys

A condition line must define a key that it's evaluating. A key is a specific part or property of the response, like response-body or status-code

| Key | Description | Example |
|--|--|--|
| `status-code` | The response status code. A numeric value | `status-code equals: 404` |
| `headers.<name>` | A specific response header | `headers.content-type contains: html` |
| `response-body` | The whole response body as a string | `response-body contains: prop1` |
| `response-body.items[0].id` | a specific key within response body, when the body is in JSON format | `response-body.items[0].id starts-with: A10` |
| `latency` | A time in milliseconds took to complete the dependecy request and evaluations | `latency less-than: 180` |

#### Operators

Every condition must define a comparison operator.

| Operator | Description | Example |
|----------|-------------|---------|
| `equals` | Exact match comparison | `status-code equals: 200` |
| `contains` | Checks if the value contains the specified substring | `response-body contains: success` |
| `matches-regex` | Matches against a regular expression pattern | `headers.content-type matches-regex: ^application/json.*$` |
| `less-than`, `less-than-or-equal` | Numeric comparison, checks if value is less than specified number | `status-code less-than: 400` |
| `greater-than`, `greater-than-or-equal` | Numeric comparison, checks if value is greater than specified number | `response-body.count greater-than: 0` |
| `starts-with` | Checks if the value starts with the specified string | `response-body.name starts-with: test_` |
| `ends-with` | Checks if the value ends with the specified string | `headers.content-type ends-with: charset=utf-8` |
| `exists` | check if a property exists or not. Expects a boolean value. Used mainly to check if a specific header provided or not, in the response | `headers.X-Custom-Header exists: true`<br>, `headers.X-Custom-Bad-Header exists: false`|

#### Modifiers/Flags

A condition may add modifiers or flags

| Modifier | Description | Example |
|----------|-------------|---------|
| `case-sensitive` | Makes string comparisons case-sensitive (default is case-insensitive). Default false | `response-body contains case-sensitive: Success` |

## Use cases

### A single request

Execute a standalone `.greq` file that contains a complete HTTP request definition with headers, body, and response validation conditions. The file processes independently without requiring any base request inheritance or dependency execution.

### A single request based on a template

Executes a single `.greq` file, but that file extends another `.greq` file. In that use case, a base file contains a set of properties that are used in this request, and the current `.greq` file only overrides or adds properties and definitions. 

Requirement: When extending another Greq file, a request line in the content section (e.g. `GET /path HTTP/1.1`) must be present. 

####  A minimal example:
Contents of the file `base-template.greq`:
```http
project: test my awesome API
is-http: true
====
POST /awesome/path
host: example.com
content-type: application/json

{"a":1,"b":2}
====
status-code equals: 200
or status-code equals: 201
```

Contents of the file `test-my-api.greq`:
```http
extends: base-template
====
POST /another/awesome/path   <--- Mandatory line

{"a":10,"b":20}
====
```

When execcuting `test-my-api.greq` file, the actual merged Greq file will look like
```http
project: test my awesome API
is-http: true
====
POST /another/awesome/path    <--- this line is overridden
host: example.com
content-type: application/json

{"a":10,"b":20}    <--- this line is overridden
====
status-code equals: 200
or status-code equals: 201
```
_Please note that only two properties were overridden in this case._

### A request that must be preceded by another request

In this use-case we need to execute another Greq file first, and only if it works and returns successful response (request succeeded, evaluations passed), we can execute the current Greq file.

This might be useful when we need to test "delete a resource" functionality. Obviously, we need to create a resource for that test first. 

Or, we want to test a functionality that is available for authorized users only. For that we need to execute a sign in request first, take the authentication token, and use it in our current request (Placeholders usage are described below).

### A simple and minimal example

The file `create-resource.greq`
```http
project: delete a resource test
====
POST /api/resources
host: example.com

{"name":"my_resource"}
====
status-code equals: 200
```

The file `delete-resource.greq`
```http
project: delete a resource test
depends-on: create-resource   <--- define a dependency
====
DELETE /api/resources/my_resource
host: example.com
====
status-code equals: 200
```

As we can clearly see, these two files are quite similar. The project name, the host, the condition. Therefore, we can combine dependency definition with extending one another. 

In this case. The `delete-resource` file could extend `create-resource` and override the request line only. As follows

The updated file `delete-resource.greq`
```
extends: create-resource
depends-on: create-resource
====
DELETE /api/resources/my_resource
====
```

## Placeholders

Greq files that define dependant requests, may use placeholders that will then be replaces with the relevant part of the response of the dependant request.

The placeholders have the following format `$(dependency.status_code)`.

The names of these placeholders must begin with `dependency.` (or the short for `dep.`), and then the actual part that we want to use.

### The available parts

| Placeholder | Description | Example |
|--|--|--|
| dependency.status-code | The status code of the dependant request | `$(dependency.status-code)` |
| dependency.headers | All the headers serialized as JSON | `$(dependency.headers)` |
| dependency.headers.<header name> | a specific response header | `$(dependency.headers.content-type)` |
| dependency.response-body | The whole response body as a string | `$(dependency.response-body)` |
| dependency.response-body.<JSON path> | The serialized internal part of the response body when it is in JSON format | `$(dependency.response-body.items[0].id)` |
| dependency.latency | The amount of milliseconds took the dependant request to complete | `$(dependency.latency)` |

### Placeholders usage example

Assume that the API that creates resources, responds with the new resource ID

```http
project: test placeholders
====
POST /api/resources
host: example.com

{"name":"my_resource"}
====
status-code equals: 200
```

The file `delete-resource.greq`. In this file we use the response body of the 'create-resource' API call, as part of the URI of the delete request.

```http
project: delete a resource test
depends-on: create-resource   <--- define a dependency
====
DELETE /api/resources/$(dependency.response-body)   <--- Use the response body in the URI
host: example.com
====
status-code equals: 200
```

