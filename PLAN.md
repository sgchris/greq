# Parsing and execution plan

## Required features

### Must features for the current version

1. Execute several files simultaneously
1. Support base requests
2. Execute dependent requests
    * Support variables based on response of the dependent request
    * Variables template $(prop)
1. Rename "base-request" to "extends"

### Future features

1. Support several request dependencies
2. Support remapping variables in the header. 
    format:
    `variables: var1_name: $(dependency_var1); var2_name: $(dependency_var2); 

    For example:
    ```
    dependency: login-request
    variables: auth_token_from_login_request: $(headers.X-Auth-Token); response_body: $(response-body)
    ```

    later on it's used as:
    ```
    POST /blog_posts
    Host: example.com
    Authentication: bearer $(auth_token_from_login_request)
    ...
    Request body
    ```
    

## Execution Plan

    

### Main algorithm

```pseudo
Parameters: 
    input file(s)
```


Execution:
```
    validate file exists and read the file

    check user-defined separator (default '=')

    parse and validate the header part
        - Ensure base-request and dependencies files exist and readable

    if there are dependant requests - execute them first
        - Keep the response (for further variables)
        - Dependency response should implement "get_var" method

    if header contains base-request
        - Call the method recursively with that file
        - Keep the parsed Greq file

    Re-parse the sections with Optional parameters
        - Greq.<section> 
        - Dependency response

    Execute the request and evaluate the response
```


### Header part

Parameters: 
```pseudo
* header lines, 
* (optional) "Extends" header object (ref)
* (optional) dependency response object (ref)
```

Execution:

```pseudo
    if dependency result provided
        replace placeholders with the variables (before the validations)

    Parse the properties and make basic validations, 
        (check unknown headers, bad format, etc.)

    if base request's header parameter provided, 
        - enrich the current header with it
        (don't include the "extends")
    otherwise if "extends" provided
        validate "extends" property (file exists)

    if dependency result provided
        replace placeholders with the variables (after the validations)
    otherwise if "dependency" provided
        (dependency wasn't executed)
        validate dependency property (file exists)
```

### Content part

Parameters:
```pseudo
* content lines
* (optional) "Extends"' content object (ref)
* (optional) dependency response object (ref)
```

Execution:

```pseudo
    if dependency result provided
        replace placeholders with the variables (before the validations)

    Parse and validate
    - The request line ("GET /some/path") is required

    if base request's content parameter provided, 
        - enrich the content with it
        (don't include the base request)

    if dependency result provided
        replace placeholders with the variables (after the validations)

```

### Footer part

Parameters:
```pseudo
* content lines
* (optional) "Extends"' footer object (ref)
* (optional) dependency response object (ref)
```

Execution:

```pseudo
    Parse and validate (format, unknown conditions)

    if base request's footer parameter provided, 
        - enrich the current footer with it

    if dependency result provided
        replace placeholders with the variables (after the validations)

```




## Stuff to consider

1. Create separate method that receives a file and returns Greq object.
    * Must be async

1. Create dependecy response struct
    * implement `get_var`

2. Rename `base-request` to `extends`



