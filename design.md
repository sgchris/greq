# Greq

## Description

*G-Req*uests is a requests manager that simultanuously executes requests defined in ".greq" files. 
Each .greq file has an HTTP request format [RFC 7230](https://datatracker.ietf.org/doc/html/rfc7230#page-19), 
with the request method, a URI, its headers and a body.
Every .greq file can have a callback for assertions

## Future features

- Login and sync requests online. Includes projects and individual requests lists

## Command examples

greq [OPTIONS] [file1.greq] [file2.greq]

Options:
    --output-dir, -o    Store the responses in a specified destination folder
                        Example: --output-dir "c:\temp\my-responses"

## Greq file spec

Greq file consist of 3 parts separated by 4 dash ("-") characters. The first part is the request metadata, the second part is the HTTP request,
and the third part is the assertions and callbacks part. 
Example:

```
project: my-project
output-folder: ../my-output-folder
-- comments start with two dash characters
output-file-name: "my-request-1-output"
----
POST /some-url/example
content-type: application/json
my-custom-header: my-value

{"my":"json", "content":"example"}
----
has-status-code: 200
or-has-status-code: 201
not-has-status-code: 500
contains: "sone content in response"
contains, regex: ^response\s+?start.*the end\.$
or-contains: "another string. The 'or' refers to the previous 'contains'"
not-contains: "unwanted string"
or-not-contains: "another unwanted string"
starts-with: "response start"
or-starts-with: "another response start"
not-starts-with: "unwanted response start"
ends-with, case-sensitive: "the end."
-- comment line here
```

Another example containing base request
```
base-request: my-base-request.greq
----
-- everything in the second section overrides the properties in the base request
POST /another-url/example
----
```

