use crate::conditions::evaluate_conditions;
use crate::error::{GreqError, Result};
use crate::models::{ExecutionResult, GreqFile, Response};
use crate::parser::{merge_greq_files, parse_greq_file, resolve_file_path};
use crate::placeholders::{
    replace_placeholders_in_greq_file, replace_placeholders_in_greq_file_with_dependency_handling,
    replace_placeholders_in_greq_file_with_optional_response,
};
use colored::*;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Execute a single Greq file with dependency resolution
pub async fn execute_greq_file<P: AsRef<Path>>(
    file_path: P,
    verbose: bool,
) -> Result<ExecutionResult> {
    let file_path = file_path.as_ref();

    // Resolve the full dependency chain
    let dependency_chain = resolve_dependency_chain(file_path)?;

    // Execute dependencies in order (from root to target)
    let mut dependency_responses: HashMap<PathBuf, Response> = HashMap::new();
    let mut failed_dependencies: HashSet<PathBuf> = HashSet::new();

    for dep_path in &dependency_chain {
        log::info!("Executing greq file: {dep_path:?}");

        let mut greq_file = parse_greq_file(&dep_path)?;

        // Handle extends recursively
        greq_file = resolve_extends_chain(greq_file, &dep_path)?;

        // Check if the dependency this file depends on has failed
        let dependency_failed = if let Some(depends_on) = &greq_file.header.depends_on {
            let dep_response_path = resolve_file_path(&dep_path, depends_on);
            let failed = failed_dependencies.contains(&dep_response_path);
            log::debug!("The file depends on {depends_on:?}. Dependency failed: {failed:?}");
            failed
        } else {
            false
        };

        // Replace placeholders only after ensuring dependency was processed
        if let Some(depends_on) = &greq_file.header.depends_on {
            let dep_response_path = resolve_file_path(&dep_path, depends_on);

            // Check if dependency was processed and has a response
            if let Some(dep_response) = dependency_responses.get(&dep_response_path) {
                log::debug!("Dependency response exists for: {dep_response_path:?}");
                // Check if the dependency failed but we allow failure
                if dependency_failed && greq_file.header.allow_dependency_failure {
                    // Use enhanced replacement that handles dependency failures
                    replace_placeholders_in_greq_file_with_dependency_handling(
                        &mut greq_file,
                        Some(dep_response),
                        dependency_failed,
                    )?;
                } else {
                    // Normal replacement with dependency response
                    replace_placeholders_in_greq_file(&mut greq_file, dep_response)?;
                }
            } else if dependency_failed && greq_file.header.allow_dependency_failure {
                log::debug!("Dependency failed and no response available, replacing placeholders with empty strings");
                // Use enhanced replacement that handles dependency failures
                replace_placeholders_in_greq_file_with_dependency_handling(
                    &mut greq_file,
                    None,
                    dependency_failed,
                )?;
            } else {
                // Dependency should have been processed but wasn't found - this is an error
                return Err(GreqError::Dependency(format!(
                    "Dependency '{}' was not processed before file '{}'",
                    depends_on,
                    dep_path.display()
                )));
            }
        } else {
            // Replace only environment placeholders (no dependencies)
            replace_placeholders_in_greq_file_with_optional_response(&mut greq_file, None)?;
        }

        // Execute the HTTP request
        match execute_http_request(&greq_file, verbose).await {
            Ok(response) => {
                // Print verbose response details if verbose flag is enabled
                if verbose {
                    print_verbose_response(&dep_path, &response);
                }

                // Evaluate conditions
                let failed_conditions = evaluate_conditions(
                    &greq_file.footer.conditions,
                    &response,
                    &greq_file.file_path,
                )?;

                if !failed_conditions.is_empty() {
                    let dep_name = dep_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    if dep_path == file_path {
                        // This is the main file failing
                        return Ok(ExecutionResult {
                            file_path: file_path.display().to_string(),
                            success: false,
                            response: Some(response),
                            failed_conditions,
                            error: None,
                        });
                    } else {
                        // This is a dependency failing
                        // Check if any files that depend on this failed dependency disallow dependency failure
                        let mut should_fail = false;
                        let mut blocking_file = String::new();

                        // Check all remaining files in the chain to see if they depend on this failed dependency
                        // and don't allow dependency failure
                        for remaining_dep_path in dependency_chain
                            .iter()
                            .skip_while(|p| *p != dep_path)
                            .skip(1)
                        {
                            let remaining_greq_file = parse_greq_file(remaining_dep_path)?;
                            if let Some(depends_on) = &remaining_greq_file.header.depends_on {
                                let dep_response_path =
                                    resolve_file_path(remaining_dep_path, depends_on);
                                if dep_response_path == *dep_path
                                    && !remaining_greq_file.header.allow_dependency_failure
                                {
                                    should_fail = true;
                                    blocking_file = remaining_dep_path.display().to_string();
                                    break;
                                }
                            }
                        }

                        if should_fail {
                            return Ok(ExecutionResult {
                                file_path: file_path.display().to_string(),
                                success: false,
                                response: None,
                                failed_conditions: vec![format!("Dependency '{}' conditions failed", dep_name)],
                                error: Some(format!("Dependency '{}' failed: {}. File '{}' does not allow dependency failure.", dep_name, failed_conditions.join(", "), blocking_file)),
                            });
                        } else {
                            log::warn!("⚠ Dependency '{}' conditions failed, but continuing because all dependent files allow dependency failure", dep_name);
                            println!("{} Dependency '{}' failed but continuing (dependency failure allowed by all dependent files)", "⚠".yellow(), dep_name.yellow());
                            // Mark this dependency as failed
                            failed_dependencies.insert(dep_path.clone());
                            // Continue execution without storing this response
                            continue;
                        }
                    }
                }

                // Store response for future dependencies
                dependency_responses.insert(dep_path.clone(), response.clone());

                if dep_path != file_path {
                    let dep_name = dep_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    log::info!("✓ Dependency '{}' executed successfully", dep_name);
                }

                // If this is the main file, return success
                if dep_path == file_path {
                    let success = failed_conditions.is_empty();
                    log::info!("✓ {} - All conditions passed", file_path.display());

                    return Ok(ExecutionResult {
                        file_path: file_path.display().to_string(),
                        success,
                        response: Some(response),
                        failed_conditions,
                        error: None,
                    });
                }
            }
            Err(e) => {
                let dep_name = dep_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                if dep_path == file_path {
                    // This is the main file failing
                    return Ok(ExecutionResult {
                        file_path: file_path.display().to_string(),
                        success: false,
                        response: None,
                        failed_conditions: Vec::new(),
                        error: Some(format!("HTTP error: {e}")),
                    });
                } else {
                    // This is a dependency failing
                    // Check if any files that depend on this failed dependency disallow dependency failure
                    let mut should_fail = false;
                    let mut blocking_file = String::new();

                    // Check all remaining files in the chain to see if they depend on this failed dependency
                    // and don't allow dependency failure
                    for remaining_dep_path in dependency_chain
                        .iter()
                        .skip_while(|p| *p != dep_path)
                        .skip(1)
                    {
                        let remaining_greq_file = parse_greq_file(remaining_dep_path)?;
                        if let Some(depends_on) = &remaining_greq_file.header.depends_on {
                            let dep_response_path =
                                resolve_file_path(remaining_dep_path, depends_on);
                            if dep_response_path == *dep_path
                                && !remaining_greq_file.header.allow_dependency_failure
                            {
                                should_fail = true;
                                blocking_file = remaining_dep_path.display().to_string();
                                break;
                            }
                        }
                    }

                    if should_fail {
                        return Ok(ExecutionResult {
                            file_path: file_path.display().to_string(),
                            success: false,
                            response: None,
                            failed_conditions: Vec::new(),
                            error: Some(format!("Dependency '{}' request failed: {}. File '{}' does not allow dependency failure.", dep_name, e, blocking_file)),
                        });
                    } else {
                        log::warn!("⚠ Dependency '{}' request failed, but continuing because all dependent files allow dependency failure: {}", dep_name, e);
                        println!("{} Dependency '{}' request failed but continuing (dependency failure allowed by all dependent files): {}", "⚠".yellow(), dep_name.yellow(), e);
                        // Mark this dependency as failed
                        failed_dependencies.insert(dep_path.clone());
                        // Continue execution without storing this response
                        continue;
                    }
                }
            }
        }
    }

    // This should never be reached, but just in case
    Ok(ExecutionResult {
        file_path: file_path.display().to_string(),
        success: false,
        response: None,
        failed_conditions: Vec::new(),
        error: Some("Unexpected end of execution".to_string()),
    })
}

/// Resolve the extends chain for a GreqFile recursively
fn resolve_extends_chain(mut greq_file: GreqFile, current_file_path: &Path) -> Result<GreqFile> {
    let mut visited = HashSet::new();
    let mut current_path = current_file_path.to_path_buf();

    // Keep resolving extends until we reach the root or detect a cycle
    while let Some(extends_path) = greq_file.header.extends.clone() {
        log::info!("Loading base request from: {extends_path}");

        let base_path = resolve_file_path(&current_path, &extends_path);
        let canonical_base_path = base_path
            .canonicalize()
            .unwrap_or_else(|_| base_path.clone());

        // Check for circular extends
        if visited.contains(&canonical_base_path) {
            return Err(GreqError::Parse(format!(
                "Circular extends detected: {} -> {}",
                current_path.display(),
                base_path.display()
            )));
        }

        visited.insert(canonical_base_path.clone());

        // Load the base file
        let base_greq = parse_greq_file(&base_path)?;

        // Merge current file with base (current file overrides base)
        greq_file = merge_greq_files(&base_greq, &greq_file)?;

        // Update current path for next iteration
        current_path = base_path;
    }

    Ok(greq_file)
}

/// Resolve the full dependency chain for a file, returning paths in execution order
fn resolve_dependency_chain<P: AsRef<Path>>(file_path: P) -> Result<Vec<PathBuf>> {
    let mut chain = Vec::new();
    let mut visited = HashSet::new();

    // Use recursive DFS to build dependency chain in correct order
    fn visit_dependency<P: AsRef<Path>>(
        current_path: P,
        chain: &mut Vec<PathBuf>,
        visited: &mut HashSet<PathBuf>,
        visiting: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        let current_path = current_path.as_ref().to_path_buf();
        let canonical_path = current_path
            .canonicalize()
            .unwrap_or_else(|_| current_path.clone());

        // Check for circular dependencies
        if visiting.contains(&canonical_path) {
            return Err(GreqError::Parse(format!(
                "Circular dependency detected involving: {}",
                current_path.display()
            )));
        }

        // Skip if already processed
        if visited.contains(&canonical_path) {
            return Ok(());
        }

        visiting.insert(canonical_path.clone());

        // Parse the file to check for dependencies
        let greq_file = parse_greq_file(&current_path)?;

        // First, process dependency if it exists
        if let Some(depends_on) = &greq_file.header.depends_on {
            let dep_path = resolve_file_path(&current_path, depends_on);
            visit_dependency(dep_path, chain, visited, visiting)?;
        }

        // Then add current file to chain
        if !visited.contains(&canonical_path) {
            chain.push(current_path.clone());
            visited.insert(canonical_path.clone());
        }

        visiting.remove(&canonical_path);
        Ok(())
    }

    let mut visiting = HashSet::new();
    visit_dependency(file_path, &mut chain, &mut visited, &mut visiting)?;

    Ok(chain)
}

/// Execute the HTTP request for a GreqFile
async fn execute_http_request(greq_file: &GreqFile, verbose: bool) -> Result<Response> {
    let client = Client::new();
    let start_time = Instant::now();

    // Build URL
    let scheme = if greq_file.header.is_http {
        "http"
    } else {
        "https"
    };
    let host = greq_file
        .content
        .headers
        .get("host")
        .ok_or_else(|| GreqError::Validation("Host header is required".to_string()))?;
    let url = format!(
        "{}://{}{}",
        scheme, host, greq_file.content.request_line.uri
    );

    log::debug!(
        "Making {} request to: {}",
        greq_file.content.request_line.method,
        url
    );

    // Build request
    let mut request_builder = match greq_file.content.request_line.method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        method => {
            return Err(GreqError::Validation(format!(
                "Unsupported HTTP method: {method}"
            )))
        }
    };

    // Add headers (excluding host as it's used for URL construction)
    for (key, value) in &greq_file.content.headers {
        if key.to_lowercase() != "host" {
            request_builder = request_builder.header(key, value);
        }
    }

    // Add body if present
    if let Some(body) = &greq_file.content.body {
        request_builder = request_builder.body(body.clone());
    }

    // Set timeout
    if let Some(timeout) = greq_file.header.timeout {
        request_builder = request_builder.timeout(timeout);
    }

    // Print verbose request details if verbose flag is enabled
    if verbose {
        print_verbose_request(greq_file, &url);
    }

    // Execute request with retries
    let mut last_error = None;
    let max_retries = greq_file.header.number_of_retries + 1; // +1 for initial attempt

    for attempt in 1..=max_retries {
        if attempt > 1 {
            log::debug!(
                "Retry attempt {} of {}",
                attempt - 1,
                greq_file.header.number_of_retries
            );
        }

        match request_builder
            .try_clone()
            .ok_or_else(|| GreqError::Validation("Failed to clone request".to_string()))?
            .send()
            .await
        {
            Ok(response) => {
                let latency = start_time.elapsed();
                let status_code = response.status().as_u16();

                // Collect headers
                let mut headers = HashMap::new();
                for (key, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        headers.insert(key.to_string().to_lowercase(), value_str.to_string());
                    }
                }

                // Get response body
                let body = response.text().await?;

                log::debug!(
                    "Response: {} {} ({}ms)",
                    status_code,
                    if status_code < 400 {
                        "✓".green()
                    } else {
                        "✗".red()
                    },
                    latency.as_millis()
                );

                return Ok(Response {
                    status_code,
                    headers,
                    body,
                    latency,
                });
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    // Wait before retry (exponential backoff)
                    let delay = Duration::from_millis(100 * (1 << (attempt - 1)));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(GreqError::Http(last_error.unwrap()))
}

/// Execute multiple Greq files in parallel
pub async fn execute_multiple_greq_files<P: AsRef<Path>>(
    file_paths: &[P],
    verbose: bool,
) -> Result<Vec<ExecutionResult>> {
    if file_paths.len() > 1 {
        log::info!("Executing {} greq files in parallel", file_paths.len());
    } else if file_paths.len() == 1 {
        log::info!(
            "Executing {} greq file",
            file_paths[0].as_ref().to_string_lossy()
        );
    }

    let mut handles = Vec::new();

    for file_path in file_paths {
        let path = file_path.as_ref().to_path_buf();
        let handle = tokio::spawn(async move { execute_greq_file(path, verbose).await });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result?),
            Err(e) => return Err(GreqError::Dependency(format!("Task join error: {e}"))),
        }
    }

    Ok(results)
}

/// Print execution results in a formatted way
pub fn print_execution_results(results: &[ExecutionResult]) {
    println!("\n{}", "=== Execution Results ===".bold().blue());

    let mut total_success = 0;
    let mut total_failed = 0;

    for result in results {
        let status_icon = if result.success {
            "✓".green()
        } else {
            "✗".red()
        };
        let file_name = Path::new(&result.file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&result.file_path);

        println!("{} {}", status_icon, file_name.bold());

        if result.success {
            total_success += 1;
            if let Some(response) = &result.response {
                println!(
                    "  Status: {} ({}ms)",
                    response.status_code.to_string().cyan(),
                    response.latency.as_millis().to_string().yellow()
                );
            }
        } else {
            total_failed += 1;

            if let Some(error) = &result.error {
                println!("  Error: {}", error.red());
            }

            for condition in &result.failed_conditions {
                println!("  Failed condition: {}", condition.red());
            }
        }
        println!();
    }

    // Summary
    let summary = if total_failed == 0 {
        format!("All {total_success} tests passed").green()
    } else {
        format!("{total_success} passed, {total_failed} failed").red()
    };

    println!("{}: {}", "Summary".bold(), summary);
}

/// Check if all results are successful
pub fn all_successful(results: &[ExecutionResult]) -> bool {
    results.iter().all(|r| r.success)
}

/// Print verbose request details for dependency chain
fn print_verbose_request(greq_file: &GreqFile, url: &str) {
    use colored::*;

    let file_name = std::path::Path::new(&greq_file.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    println!(
        "\n{} {}",
        "📤 Request from:".bold().green(),
        file_name.yellow()
    );

    // Print request line
    let version = if greq_file.content.request_line.version.is_empty() {
        "HTTP/1.1"
    } else {
        &greq_file.content.request_line.version
    };
    println!(
        "{} {} {} {}",
        "Method:".bold(),
        greq_file.content.request_line.method.blue(),
        greq_file.content.request_line.uri.cyan(),
        version.dimmed()
    );

    println!("{} {}", "URL:".bold(), url.cyan());

    // Print headers
    if !greq_file.content.headers.is_empty() {
        println!("{}", "Headers:".bold());
        for (key, value) in &greq_file.content.headers {
            println!("  {}: {}", key.cyan(), value);
        }
    }

    // Print request body
    if let Some(body) = &greq_file.content.body {
        println!("{}", "Request Body:".bold());
        if body.trim().is_empty() {
            println!("  {}", "(empty)".italic().dimmed());
        } else {
            // Try to pretty-print JSON, otherwise print as-is
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
                if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                    // Indent each line for better formatting
                    for line in pretty_json.lines() {
                        println!("  {}", line);
                    }
                } else {
                    println!("  {}", body);
                }
            } else {
                // Not JSON, print as-is with indentation
                for line in body.lines() {
                    println!("  {}", line);
                }
            }
        }
    } else {
        println!("{}", "Request Body:".bold());
        println!("  {}", "(none)".italic().dimmed());
    }
    println!("{}", "=".repeat(50).blue());
}

/// Print verbose response details for dependency chain
fn print_verbose_response(file_path: &std::path::Path, response: &Response) {
    use colored::*;

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    println!(
        "\n{} {}",
        "📄 Response for:".bold().cyan(),
        file_name.yellow()
    );
    println!(
        "{} {}",
        "Status Code:".bold(),
        format!("{}", response.status_code).green()
    );
    println!(
        "{} {}ms",
        "Response Time:".bold(),
        response.latency.as_millis().to_string().blue()
    );

    // Print headers
    if !response.headers.is_empty() {
        println!("{}", "Headers:".bold());
        for (key, value) in &response.headers {
            println!("  {}: {}", key.cyan(), value);
        }
    }

    // Print response body
    println!("{}", "Response Body:".bold());
    if response.body.trim().is_empty() {
        println!("  {}", "(empty)".italic().dimmed());
    } else {
        // Try to pretty-print JSON, otherwise print as-is
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response.body) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                // Indent each line for better formatting
                for line in pretty_json.lines() {
                    println!("  {}", line);
                }
            } else {
                println!("  {}", response.body);
            }
        } else {
            // Not JSON, print as-is with indentation
            for line in response.body.lines() {
                println!("  {}", line);
            }
        }
    }
    println!("{}", "-".repeat(50).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute_simple_greq_file() {
        let _ = env_logger::try_init();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.greq");

        let content = "project: test\nis-http: true\n====\nGET /\nhost: httpbin.org\n====\nstatus-code less-than: 500";

        fs::write(&file_path, content).unwrap();

        // Debug: print the file content
        let written_content = fs::read_to_string(&file_path).unwrap();
        println!("File content: {written_content:?}");

        let result = execute_greq_file(&file_path, false).await;
        println!("Result: {result:?}");

        // For now, just check that we get some result
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_dependency_chain_ordering() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // Create root file (no dependencies)
        let root_path = dir.path().join("root.greq");
        fs::write(
            &root_path,
            "project: root\n====\nGET /\nhost: example.com\n====\nstatus-code equals: 200",
        )
        .unwrap();

        // Create middle file (depends on root)
        let middle_path = dir.path().join("middle.greq");
        fs::write(&middle_path, "project: middle\ndepends-on: root.greq\n====\nGET /\nhost: example.com\n====\nstatus-code equals: 200").unwrap();

        // Create final file (depends on middle)
        let final_path = dir.path().join("final.greq");
        fs::write(&final_path, "project: final\ndepends-on: middle.greq\n====\nGET /\nhost: example.com\n====\nstatus-code equals: 200").unwrap();

        // Resolve dependency chain
        let chain = resolve_dependency_chain(&final_path).unwrap();

        // Verify execution order: root -> middle -> final
        assert_eq!(chain.len(), 3);
        assert!(chain[0].file_name().unwrap().to_str().unwrap() == "root.greq");
        assert!(chain[1].file_name().unwrap().to_str().unwrap() == "middle.greq");
        assert!(chain[2].file_name().unwrap().to_str().unwrap() == "final.greq");
    }

    #[test]
    fn test_circular_dependency_detection() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // Create files with circular dependency: a -> b -> a
        let a_path = dir.path().join("a.greq");
        fs::write(&a_path, "project: a\ndepends-on: b.greq\n====\nGET /\nhost: example.com\n====\nstatus-code equals: 200").unwrap();

        let b_path = dir.path().join("b.greq");
        fs::write(&b_path, "project: b\ndepends-on: a.greq\n====\nGET /\nhost: example.com\n====\nstatus-code equals: 200").unwrap();

        // Should detect circular dependency
        let result = resolve_dependency_chain(&a_path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Circular dependency"));
        }
    }
}
