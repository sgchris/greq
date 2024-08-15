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

    --log-file, -l      Full path to the log file. By default the log is stored in a temporary
                        folder (e.g. C:\Temp) and called greq.log

## Greq file spec

Greq file consist of 3 parts separated by 4 equal ("=") characters (at least 4, can be more).
The first part is the request metadata, the second part is the HTTP request, and the third part consists of assertions. 
Example:

```
project: my-project
output-folder: ../my-output-folder
-- comments start with two dash characters
output-file-name: "my-request-1-output"
certificate: c:\certs\my-cert.pfx
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
response equals: "full response content"
response contains: "some content in response"
response contains regex: ^response\s+?start.*the end\.$
or response contains: "another string. The 'or' refers to the previous 'contains'"
not response ontains: "unwanted string"
and not contains: "another unwanted string"
header exists: "custom-header-name"
header equals: "custom-header-name", "some custom header value"
header contains: "custom-header-name", "some sub-string"
header contains regex: "custom-header-name", "some.*reg[ex]+"
response starts-with: "response start"
or response starts-with: "another response start"
not response starts-with: "unwanted response start"
response ends-with case-sensitive: "the end."
-- comment line here
```

### Reserved properties
- Header
1. project - the name of the project
2. output-folder - Where to store the raw response
3. output-file-name - The name of the output file. Default is the name of the current file with extention ".response"
4. certificate - Absolute path to the certificate
5. base-request - reference to GReq file to be used as a base/reference file. The current file with use the header, contents and the footer
from the base file. Everything defined in this file will extend/override the values provided in the base GReq file.

- Content
1. Standard HTTP request protocol
+ Request method and the URI.
+ Headers
+ Request content (After two "\r\n" EOL characters, as the standard HTTP protocol)

- Footer
conditions are defined in the following format:
[and/or] [not] <property> [exists][contains][equals][starts-with][ends-with] [regex] [case-sensitive]: [value/sub-string][, value/sub-string]

Available properties
1. status-code 
2. header
2. response

Notes:
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

