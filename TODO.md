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

15. support return code from the app. 0 for success, 1 for failure, 2 for error, etc. Check what's usually required for CLI apps.

16. input file parameter - check relative and absolute paths. If relative, use the current working directory as a base.

17. Merge the request body with the base request body if the `merge-json-body` header is set to true.

18. Add optional retries. `number-of-retries` property in the header

20. fix error messages

21. add conditional dependency

22. add the option to export variables (for dependant requests)

23. Check if a header is a numeric value. Support greater|less-than in this case

24. Add response body json parsing support. A condition like `response-body.project.myProj.id starts-with: myproj`. Check the correct format for `pointer` method. Does it accept dots or just slashes.

25. check that `is-http` can be extended (convert to `Option<bool>`)

26. add `override-request-body` property to the header. When you want to extend a greq, but without a request body

27. add `latency` to footer conditions

28. add `exists` to footer conditions. Used to test that some headers exist

29. For 'delete' requests don't send the request body. (check other request methods)

30. Change placehoders to use dashes rather than underscores for consistency. `status-code` instead of `status_code`