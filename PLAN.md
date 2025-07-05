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

The basic algorithm

```pseudo
start:

    validate file exists and read the file

    check user-defined separator (default '=')

    parse and validate the header part
        Ensure base-request and dependencies files exist and readable

    if there are dependant requests - execute them first
        Generate the variables

    if header contains base-request
        Call the method recursively with the base-request file

    then parse the base-request part


```
