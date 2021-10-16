use warp::{Filter, Rejection, Reply, filters::BoxedFilter};
use prometheus::{self, IntCounterVec};

lazy_static! {
    pub static ref ECHO_COUNT: IntCounterVec = register_int_counter_vec!(
        "echo_total",
        "echo count",
        &["method"]
    )
    .unwrap();
}

pub async fn collect_metrics() -> String {
    use prometheus::{Encoder, TextEncoder};

    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    // Gather the metrics.
    let metric_families = prometheus::gather();
    // Encode them to send.
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}

async fn metrics_response() -> Result<impl Reply, Rejection> {
    Ok(collect_metrics().await)
}

pub fn metrics_handler() -> BoxedFilter<(impl Reply,)> {
    warp::path!("metrics")
        .and(warp::get())
        .and_then(metrics_response)
        .boxed()
}
