use warp::{Filter, Rejection, Reply, filters::BoxedFilter};
use std::collections::HashMap;
use tracing::*;
use askama::Template;
use serde::{Deserialize, Serialize};
use crate::api;

#[derive(Serialize)]
struct ExpensiveResult {
    server: String,
    prime_limit: u32,
    fib_length: u32,
    computation_result: String,
    primes_count: usize,
    fibonacci_value: u128,
    execution_time_ms: u64,
}

#[derive(Deserialize, Debug)]
struct ExpensiveQuery {
    prime_limit: u32,
    fib_length: u32,
}

#[derive(Debug)]
struct ValidationError(String);
impl warp::reject::Reject for ValidationError {}

#[instrument]
async fn matrix_multiplication() -> Vec<Vec<i32>> {
    info!("Starting matrix multiplication");
    let size = 100;
    let matrix_a = vec![vec![1; size]; size];
    let matrix_b = vec![vec![2; size]; size];
    let mut result = vec![vec![0; size]; size];
    
    for i in 0..size {
        for j in 0..size {
            for k in 0..size {
                result[i][j] += matrix_a[i][k] * matrix_b[k][j];
            }
        }
    }
    info!("Matrix multiplication completed");
    result
}

#[instrument]
async fn prime_calculation(limit: u32) -> Vec<u32> {
    info!(limit, "Starting prime number calculation");
    let mut primes = Vec::new();
    
    for num in 2..=limit {
        let mut is_prime = true;
        for i in 2..=(num as f64).sqrt() as u32 {
            if num % i == 0 {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            primes.push(num);
        }
    }
    info!(count = primes.len(), "Prime calculation completed");
    primes
}

#[instrument]
async fn fibonacci_sequence(n: u32) -> Vec<u128> {
    info!(n, "Generating fibonacci sequence");
    let mut fib = vec![0, 1];
    
    for i in 2..n {
        let next = fib[(i-1) as usize] + fib[(i-2) as usize];
        fib.push(next);
    }
    info!(length = fib.len(), "Fibonacci sequence generated");
    fib
}

#[instrument]
async fn some_expensive_computation(prime_limit: u32, fib_length: u32) -> (String, usize, u128) {
    info!(prime_limit, fib_length, "Starting expensive computation pipeline");
    
    let matrix_span = span!(Level::INFO, "matrix_computation");
    let _matrix_guard = matrix_span.enter();
    let _matrix_result = matrix_multiplication().await;
    drop(_matrix_guard);
    
    let prime_span = span!(Level::INFO, "prime_computation");
    let _prime_guard = prime_span.enter();
    let primes = prime_calculation(prime_limit).await;
    drop(_prime_guard);
    
    let fib_span = span!(Level::INFO, "fibonacci_computation");
    let _fib_guard = fib_span.enter();
    let fibonacci = fibonacci_sequence(fib_length).await;
    drop(_fib_guard);
    
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    let fib_index = (fib_length - 1) as usize;
    let fib_value = *fibonacci.get(fib_index).unwrap_or(&0);
    let result = format!(
        "Computed {} primes (limit: {}), fibonacci[{}] = {}", 
        primes.len(),
        prime_limit,
        fib_index,
        fib_value
    );
    
    info!(result = %result, "Expensive computation completed");
    (result, primes.len(), fib_value)
}


#[instrument]
async fn expensive_get_handler() -> Result<impl Reply, Rejection> {
    let server = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    let template = api::ExpensiveTemplate::new(server);
    let html = template.render().map_err(|e| {
        error!("Template render error: {}", e);
        warp::reject::not_found()
    })?;
    Ok(warp::reply::html(html))
}

#[instrument]
async fn expensive_post_handler(form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let server = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    
    let prime_limit = match form.get("prime_limit").and_then(|s| s.parse::<u32>().ok()) {
        Some(val) if val >= 2 && val <= 10000 => val,
        Some(_) => {
            let template = api::ExpensiveTemplate::with_error(
                server,
                None,
                None,
                "Prime limit must be between 2 and 10,000".to_string()
            );
            let html = template.render().map_err(|_| warp::reject::not_found())?;
            return Ok(warp::reply::html(html));
        },
        None => {
            let template = api::ExpensiveTemplate::with_error(
                server,
                None,
                None,
                "Prime limit is required".to_string()
            );
            let html = template.render().map_err(|_| warp::reject::not_found())?;
            return Ok(warp::reply::html(html));
        }
    };
    
    let fib_length = match form.get("fib_length").and_then(|s| s.parse::<u32>().ok()) {
        Some(val) if val >= 2 && val <= 100 => val,
        Some(_) => {
            let template = api::ExpensiveTemplate::with_error(
                server,
                Some(prime_limit),
                None,
                "Fibonacci length must be between 2 and 100".to_string()
            );
            let html = template.render().map_err(|_| warp::reject::not_found())?;
            return Ok(warp::reply::html(html));
        },
        None => {
            let template = api::ExpensiveTemplate::with_error(
                server,
                Some(prime_limit),
                None,
                "Fibonacci length is required".to_string()
            );
            let html = template.render().map_err(|_| warp::reject::not_found())?;
            return Ok(warp::reply::html(html));
        }
    };
    
    info!(prime_limit, fib_length, "Handling expensive computation request");
    let (result, _, _) = some_expensive_computation(prime_limit, fib_length).await;
    info!("Expensive computation request completed");
    
    let template = api::ExpensiveTemplate::with_result(server, prime_limit, fib_length, result);
    let html = template.render().map_err(|_| warp::reject::not_found())?;
    Ok(warp::reply::html(html))
}

#[instrument]
async fn expensive_json_handler(query: ExpensiveQuery) -> Result<impl Reply, Rejection> {
    let prime_limit = query.prime_limit;
    let fib_length = query.fib_length;
    
    // Validate parameters
    if prime_limit < 2 || prime_limit > 10000 {
        return Err(warp::reject::custom(ValidationError("Prime limit must be between 2 and 10,000".to_string())));
    }
    
    if fib_length < 2 || fib_length > 100 {
        return Err(warp::reject::custom(ValidationError("Fibonacci length must be between 2 and 100".to_string())));
    }
    
    let server = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    
    info!(prime_limit, fib_length, "Handling JSON expensive computation request");
    let start_time = std::time::Instant::now();
    let (computation_result, primes_count, fibonacci_value) = some_expensive_computation(prime_limit, fib_length).await;
    let execution_time_ms = start_time.elapsed().as_millis() as u64;
    info!("JSON expensive computation request completed");
    
    let result = ExpensiveResult {
        server,
        prime_limit,
        fib_length,
        computation_result,
        primes_count,
        fibonacci_value,
        execution_time_ms,
    };
    
    Ok(warp::reply::json(&result))
}

async fn handle_validation_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(validation_error) = err.find::<ValidationError>() {
        let error_message = &validation_error.0; // Access the String field
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": error_message
            })),
            warp::http::StatusCode::BAD_REQUEST
        ));
    }
    Err(err)
}


pub fn expensive_handler() -> BoxedFilter<(impl Reply,)> {
    let get_json_route = warp::path::end()
        .and(warp::get())
        .and(warp::header::exact_ignore_case("content-type", "application/json"))
        .and(warp::query::<ExpensiveQuery>())
        .and_then(expensive_json_handler)
        .recover(handle_validation_error);
    
    let get_html_route = warp::path::end()
        .and(warp::get())
        .and_then(expensive_get_handler);
    
    let post_route = warp::path::end()
        .and(warp::post())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(expensive_post_handler);
    
    get_json_route.or(get_html_route).or(post_route).boxed()
}