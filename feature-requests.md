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

6. Performance and load tests
    Example:
    ```yaml
    project: User Registration Load Test
    performance-mode: true
    concurrent-users: 50
    ramp-up-time: 30s
    test-duration: 5m

    ====
    POST /api/users HTTP/1.1
    # ... request details
    ====

    latency percentile-95 less-than: 500
    throughput greater-than: 100
    error-rate less-than: 1%
    ```

7. Scheduling and repeat. Add property like `repeat-every: 5 minutes`, `repeat-every: 15 seconds`, `repeat-every: 150 milliseconds`