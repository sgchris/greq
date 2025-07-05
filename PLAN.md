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

## Execution Plan

### Main algorithm

```pseudo

Parameters: input file(s)

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

```pseudo

Parameters: 
* header lines, 
* (optional) base request's header object (ref)
* (optional) dependency response object (ref)

Execution:

    Parse the properties and make basic validations, 
        (check unknown headers, bad format, etc.)

    If base request's header parameter provided, enrich with it
    (don't include the base request, it should be loaded recursively)

    Validate "extends" and "dependency"
        Check that files exist and readable

```

### Content part

```pseudo
    Parse the properties and make basic validations, like check unknown headers, bad format, etc.

    If base request header provided, enrich with it

    Validate "extends" and "dependency"
        Check that files exist and readable

```




## Stuff to consider

1. Create separate method that receives a file and returns Greq object.
    * Must be async

1. Create dependecy response struct
    * implement `get_var`

2. Rename `base-request` to `extends`



