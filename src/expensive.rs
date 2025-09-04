use warp::{Filter, Rejection, Reply, filters::BoxedFilter};
use std::collections::HashMap;
use tracing::*;
use askama::Template;
use crate::api;

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
async fn fibonacci_sequence(n: u32) -> Vec<u64> {
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
async fn some_expensive_computation(prime_limit: u32, fib_length: u32) -> String {
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
    let result = format!(
        "Computed {} primes (limit: {}), fibonacci[{}] = {}", 
        primes.len(),
        prime_limit,
        fib_index,
        fibonacci.get(fib_index).unwrap_or(&0)
    );
    
    info!(result = %result, "Expensive computation completed");
    result
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
    let result = some_expensive_computation(prime_limit, fib_length).await;
    info!("Expensive computation request completed");
    
    let template = api::ExpensiveTemplate::with_result(server, prime_limit, fib_length, result);
    let html = template.render().map_err(|_| warp::reject::not_found())?;
    Ok(warp::reply::html(html))
}

pub fn expensive_handler() -> BoxedFilter<(impl Reply,)> {
    let get_route = warp::path::end()
        .and(warp::get())
        .and_then(expensive_get_handler);
    
    let post_route = warp::path::end()
        .and(warp::post())
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(expensive_post_handler);
    
    get_route.or(post_route).boxed()
}