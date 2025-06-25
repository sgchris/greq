# TODO

1. Convert CLI parameters to raw parameters with optional named parameters. For example:
`# greq file1.greq file2.greq --skip-evaluation true`

2. Add support for multiple files in the CLI. For example:
`# greq file1.greq file2.greq --skip-evaluation true`

3. Add support for files patterns in the CLI. For example:
`# greq *.greq --skip-evaluation true`

4. avoid dependency loop (base request)

5. Add standard logger instead of print statements.

6. Update the content port when "is-http" is set to true in the request. By default it's 443

7. add 'merge-json-body' boolean header. Default to false.
In case when 'base-request' is set, the body (if it's a JSON) can be either overwritten or merged with the base request body.

8. Add regex support for 'equals'

9. Add support for using response of the dependant request in the current request.

10. Convert greq_response.status_code to StatusCode enum.

11. consider adding `timeout` and `description` to the request header.

12. Check the case when is_http is not compatible with the port. (E.g. true and 80, false and 443)

13. add properties to the header: override headers, override json body.

14. allow missing required values when base request is set.
add many tests for this case.
