use std::convert::Infallible;
use chrono::Utc;

use std::time::Duration;
use futures::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use warp::{Reply, sse::Event};

fn sse_counter(counter: u64) ->  Result<Event, Infallible> {
    let event = Event::default()
        .data(Utc::now().to_rfc3339())
        .id(counter.to_string())
        .retry(Duration::from_millis(500));
    Ok(event)
}

pub async fn sse_stream() -> Result<impl Reply, Infallible> {
    let mut counter: u64 = 0;
    let interval = interval(Duration::from_secs(5));
    let stream = IntervalStream::new(interval);
    let event_stream = stream.map(move |_| {
        counter += 1;
        sse_counter(counter)
    });
    // reply using server-sent events
    let stream = warp::sse::keep_alive()
        .interval(Duration::from_secs(5))
        .text("tick".to_string())
        .stream(event_stream);

    Ok(warp::sse::reply(stream))
}

