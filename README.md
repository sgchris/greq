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

--show-response - Show the response content in the console
--skip-evaluation - Skip the evaluation of the response. Useful for debugging purposes.
--show-request-only - Show the request content in the console without sending it.

## Greq file spec

- Greq file consist of 3 parts separated by 4 equal ("=") characters (at least 4, can be more). 
The first part is the request metadata, the second part is the raw HTTP request, and the third part consists of assertions. 

- The default separator is the '=' char, but it can be user-defined by adding the "separator" property in the header part. e.g. "separator: *".


### Example:

```
project: my-project
-- comments start with two dash characters
====
POST /some-url/example
host: example.com 
content-type: application/json
my-custom-header: my-value

{"my":"json", "content":"example"}
====
status-code equals: 200
or status-code: 201
not status-code: 500
response-body equals: full response content
response-body contains: some content in response
headers.custom-header matches-regex: ^response\s+?start.*the end\.$
or response-body contains: another string. The 'or' refers to the previous 'contains'"
not response-body ontains: unwanted string"
and not contains: another unwanted string"
header exists: custom-header-name
response-body starts-with: response start
or response-body starts-with: another response start
not response-body starts-with: unwanted response start
response-body ends-with case-sensitive: the end.
-- comment line here
```

### Important notes

- The values in the header and the footer **must not** be wrappered in quotes, unless you want to use them as a string.
- The values in the header and the footer **do not support multi-line**. all the values must be in a single line.

## Reserved properties

### Header

1. *project* - the name of the project
2. *certificate* - Absolute path to the certificate
3. *base-request-path* - reference to GReq file to be used as a base/reference file. The current file with use the header, contents and the footer
from the base file. Everything defined in this file will extend/override the values provided in the base GReq file.
4. *depends-on* - Execute another GReq file before executing this one. (For example, if this request performs delete or update request, you need to create the resource first. You can do that in another GReq file and mention it here)

### Content

Standard HTTP request protocol

+ Request method and the URI.
+ Headers
+ Request content (After two "\r\n" EOL characters, as the standard HTTP protocol)

### Footer

conditions are defined in the following format:

```
[and/or] [not] *<property>* [exists][contains][equals][starts-with][ends-with] [regex] [case-sensitive]: [value/sub-string][, value/sub-string]
```

*Available properties*

1. status-code 
2. header
2. response-body

*Notes*

1. And/Or are related to the previous line only



### Another example containing base request

```
base-request: my-base-request.greq
====
-- everything in the second section overrides the properties in the base request
POST /another-url/example
my-custom-header: another value different from the previous one
====
status-code equals: 400
or status-code equals: 404
```

