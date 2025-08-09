# Features requests

1. Add authentication with certificates
    - Load from an absolute path
    - Load from a cloud's key vault
    - provide password
        * raw
        * passwords file as a parameter

2. Add `allow-dependency-failure` property to the header

3. Add "if" statements
    - Could be combined with `allow-dependency-failure`

4. Add Extensions mechanism. Extensions examples:
    - Send metrics and logs
    - Call callback API

5. Add support for parameters files
    - Allow several parameters file for the same Greq file to be executed one by one
    - Add variable to distinguish between the parameter files, like `$(parameters.hostname)`
