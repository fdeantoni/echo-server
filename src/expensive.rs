use warp::{Filter, Rejection, Reply, filters::BoxedFilter};
use std::collections::HashMap;
use tracing::*;

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
async fn expensive_response(params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let prime_limit = params
        .get("prime_limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
        .max(2)
        .min(10000);
    
    let fib_length = params
        .get("fib_length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
        .max(2)
        .min(100);
    
    info!(prime_limit, fib_length, "Handling expensive computation request");
    let result = some_expensive_computation(prime_limit, fib_length).await;
    info!("Expensive computation request completed");
    Ok(result)
}

pub fn expensive_handler() -> BoxedFilter<(impl Reply,)> {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(expensive_response)
        .boxed()
}