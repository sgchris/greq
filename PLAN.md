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

### Greq algorithm (will be called from main)

```pseudo
Parameters: 
    input file(s) - vec of strings
    Parse Only - boolean, default false
```

Execution:
```

main method:
    validate file exists and read the file

    check user-defined separator (default '=')

    parse and validate the header part

    If provided "extends" or "dependency"
        - Ensure files exist and readable
        - run the following processes simultaneously 
            * load 'base' 
                - call recursively with 'Parse Only' true
                - Keep the Greq object of the base request
            * execute 'dependency' (only if 'Parse Only' is false)
                - call recursively with 'Parse Only' false
                - Keep the execution response object

    if "extends" was provided re-parse the header part

    parse the content and the footer (with base and dependency results if provided)

    If Parse-Only is false
        - Send the request and evaluate the response
        - Return execution response
    Otherwise
        - return the parsed Greq object
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

# Stuff to consider

1. Create separate method that receives a file and returns Greq object.
    * Must be async

1. Create dependecy response struct
    * Contain execution result (0 - success, otherwise it's an error code)
    * implement `get_var`

2. Rename `base-request` to `extends`

3. Execution results - consider using sysexits crate
    - 0: Success
    - 1: General error
    - 64: Command line usage error
    - 65: Data format error
    - 69: Service unavailable
    - 74: Input/output error




