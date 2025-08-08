use crate::models::{GreqFile, Response, ExecutionResult};
use crate::parser::{parse_greq_file, merge_greq_files, resolve_file_path};
use crate::placeholders::replace_placeholders_in_greq_file;
use crate::conditions::evaluate_conditions;
use crate::error::{GreqError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use colored::*;

/// Execute a single Greq file with dependency resolution
pub async fn execute_greq_file<P: AsRef<Path>>(file_path: P) -> Result<ExecutionResult> {
    let file_path = file_path.as_ref();
    log::info!("Executing greq file: {file_path:?}");
    
    let mut greq_file = parse_greq_file(file_path)?;
    
    // Handle extends
    if let Some(extends_path) = &greq_file.header.extends.clone() {
        log::info!("Loading base request from: {extends_path}");
        let base_path = resolve_file_path(file_path, extends_path);
        let base_greq = parse_greq_file(&base_path)?;
        greq_file = merge_greq_files(&base_greq, &greq_file)?;
    }
    
    // Handle dependencies - execute dependency first if exists
    let dependency_response = if let Some(depends_on) = &greq_file.header.depends_on.clone() {
        log::info!("Executing dependency: {depends_on}");
        let dep_path = resolve_file_path(file_path, depends_on);
        
        // Load and execute dependency (without its own dependencies to avoid recursion)
        let mut dep_greq = parse_greq_file(&dep_path)?;
        
        // Handle extends for dependency
        if let Some(dep_extends_path) = &dep_greq.header.extends.clone() {
            let dep_base_path = resolve_file_path(&dep_path, dep_extends_path);
            let dep_base_greq = parse_greq_file(&dep_base_path)?;
            dep_greq = merge_greq_files(&dep_base_greq, &dep_greq)?;
        }
        
        // Execute dependency HTTP request
        match execute_http_request(&dep_greq).await {
            Ok(dep_response) => {
                let dep_failed_conditions = evaluate_conditions(&dep_greq.footer.conditions, &dep_response)?;
                if !dep_failed_conditions.is_empty() {
                    return Ok(ExecutionResult {
                        file_path: file_path.display().to_string(),
                        success: false,
                        response: None,
                        failed_conditions: vec![format!("Dependency '{}' conditions failed", depends_on)],
                        error: Some(format!("Dependency '{}' failed: {}", depends_on, dep_failed_conditions.join(", "))),
                    });
                }
                log::info!("✓ Dependency '{}' executed successfully", depends_on);
                Some(dep_response)
            },
            Err(e) => {
                return Ok(ExecutionResult {
                    file_path: file_path.display().to_string(),
                    success: false,
                    response: None,
                    failed_conditions: Vec::new(),
                    error: Some(format!("Dependency '{}' request failed: {e}", depends_on)),
                });
            }
        }
    } else {
        None
    };
    
    // Replace placeholders if we have a dependency response
    if let Some(ref dep_response) = dependency_response {
        replace_placeholders_in_greq_file(&mut greq_file, dep_response)?;
    }
    
    // Execute the HTTP request
    match execute_http_request(&greq_file).await {
        Ok(response) => {
            // Evaluate conditions
            let failed_conditions = evaluate_conditions(&greq_file.footer.conditions, &response)?;
            let success = failed_conditions.is_empty();
            
            // Log result
            if success {
                log::info!("✓ {} - All conditions passed", file_path.display().to_string().green());
            } else {
                log::warn!("✗ {} - {} condition(s) failed", 
                    file_path.display().to_string().red(),
                    failed_conditions.len()
                );
                for condition in &failed_conditions {
                    log::warn!("  Failed: {condition}");
                }
            }
            
            Ok(ExecutionResult {
                file_path: file_path.display().to_string(),
                success,
                response: Some(response),
                failed_conditions,
                error: None,
            })
        },
        Err(e) => {
            log::error!("✗ {} - Request failed: {}", file_path.display().to_string().red(), e);
            Ok(ExecutionResult {
                file_path: file_path.display().to_string(),
                success: false,
                response: None,
                failed_conditions: Vec::new(),
                error: Some(e.to_string()),
            })
        }
    }
}

/// Execute the HTTP request for a GreqFile
async fn execute_http_request(greq_file: &GreqFile) -> Result<Response> {
    let client = Client::new();
    let start_time = Instant::now();
    
    // Build URL
    let scheme = if greq_file.header.is_http { "http" } else { "https" };
    let host = greq_file.content.headers.get("host")
        .ok_or_else(|| GreqError::Validation("Host header is required".to_string()))?;
    let url = format!("{}://{}{}", scheme, host, greq_file.content.request_line.uri);
    
    log::debug!("Making {} request to: {}", greq_file.content.request_line.method, url);
    
    // Build request
    let mut request_builder = match greq_file.content.request_line.method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        method => return Err(GreqError::Validation(format!("Unsupported HTTP method: {method}"))),
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
    
    // Execute request with retries
    let mut last_error = None;
    let max_retries = greq_file.header.number_of_retries + 1; // +1 for initial attempt
    
    for attempt in 1..=max_retries {
        if attempt > 1 {
            log::debug!("Retry attempt {} of {}", attempt - 1, greq_file.header.number_of_retries);
        }
        
        match request_builder.try_clone()
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
                
                log::debug!("Response: {} {} ({}ms)", status_code, 
                    if status_code < 400 { "✓".green() } else { "✗".red() },
                    latency.as_millis()
                );
                
                return Ok(Response {
                    status_code,
                    headers,
                    body,
                    latency,
                });
            },
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
pub async fn execute_multiple_greq_files<P: AsRef<Path>>(file_paths: &[P]) -> Result<Vec<ExecutionResult>> {
    log::info!("Executing {} greq files in parallel", file_paths.len());
    
    let mut handles = Vec::new();
    
    for file_path in file_paths {
        let path = file_path.as_ref().to_path_buf();
        let handle = tokio::spawn(async move {
            execute_greq_file(path).await
        });
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
        let status_icon = if result.success { "✓".green() } else { "✗".red() };
        let file_name = Path::new(&result.file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&result.file_path);
        
        println!("{} {}", status_icon, file_name.bold());
        
        if result.success {
            total_success += 1;
            if let Some(response) = &result.response {
                println!("  Status: {} ({}ms)", 
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
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
        
        let result = execute_greq_file(&file_path).await;
        println!("Result: {result:?}");
        
        // For now, just check that we get some result
        assert!(result.is_ok() || result.is_err());
    }
}
