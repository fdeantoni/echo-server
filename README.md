# Echo Server

It's purpose is similar to that of [jmalloc/echo-server](https://github.com/jmalloc/echo-server) except
that it has the following paths defined:

- [/](http://localhost:9000/) returns a simple HTML page with the headers received and some extra details
- [/echo](http://localhost:9000/echo) returns the headers etc in JSON format. Supports any HTTP method.
- /ws provides a websocket connection to an echo server
- /sse provides an event source connection to a tick timer sending the time every 5 seconds.

Any other path will still result in a JSON response with headers etc except that the HTTP status code
returned will be HTTP 404 NOT FOUND.

Beyond this it also supports prometheus metrics at [/metrics](http://127.0.0.1:9000/metrics).

Example GET:

```console
$ curl -X GET http://127.0.0.1:9000/echo
{"source":"127.0.0.1:57730","method":"GET","headers":[["host","127.0.0.1:9000"],["user-agent","curl/7.64.1"],["accept","*/*"]],"path":"/echo"}
```

You can do a GET, POST, or DELETE at any path:

```console
$ curl -X POST --data "hello there" http://127.0.0.1:9000/echo/some/other/path
{"source":"127.0.0.1:57884","method":"POST","headers":[["host","127.0.0.1:9000"],["user-agent","curl/7.64.1"],["accept","*/*"],["content-length","11"],["content-type","application/x-www-form-urlencoded"]],"path":"/echo/some/other/path","body":"hello there"}
```

For websocket connections use can use [websocat](https://github.com/vi/websocat) to test:

```console
$ websocat ws://127.0.0.1:9000/ws
hello
hello
```

For SSE connections you can test using your browsers JavaScript console:

```javascript
const sse = new EventSource('http://localhost:9000/sse');
sse.onmessage = console.log
```

## OpenTelemetry

This echo server uses open tokio tracing and can send logs/metrics/traces to an OTLP endpoint. To do so, simply define the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable.

The OpenTelemetry logger can be configured with the following additional environment variables:

- `OTEL_EXPORTER_OTLP_ENDPOINT`: The endpoint to send OTLP data to.
- `OTEL_SERVICE_NAME`: The name of the service.
- `OTEL_SERVICE_NAMESPACE`: The namespace of the service.
- `OTEL_SERVICE_VERSION`: The version of the service.
- `OTEL_SERVICE_INSTANCE_ID`: The instance ID of the service.
- `OTEL_DEPLOYMENT_ENVIRONMENT`: The deployment environment of the service.

To simulate some traces, there is an endpoint `/expensive` that will execute some nested functions to generate a trace.

### Expensive Function Details

The `/expensive` endpoint provides a comprehensive demonstration of OpenTelemetry tracing through a multi-stage computational pipeline:

- **GET /expensive**: Returns an interactive form to configure the computation parameters
- **GET /expensive?prime_limit=N&fib_length=M** (with Content-Type: application/json): Returns computation results in JSON format
- **POST /expensive**: Executes the expensive computation with user-provided parameters

The computation pipeline includes three instrumented operations:
1. **Matrix Multiplication**: Performs a 100x100 matrix multiplication operation
2. **Prime Number Calculation**: Computes all prime numbers up to a specified limit (2-10,000)
3. **Fibonacci Sequence Generation**: Generates a Fibonacci sequence of specified length (2-100)

Each operation is wrapped in its own tracing span and includes detailed logging with structured data. The pipeline also includes a 500ms artificial delay to demonstrate async operation tracing. All functions are instrumented with the `#[instrument]` macro to automatically generate trace spans with input parameters and execution context.

#### JSON API Usage

To use the JSON endpoint, send a GET request with the appropriate query parameters and Content-Type header:

```console
$ curl -H "Content-Type: application/json" "http://127.0.0.1:9000/expensive?prime_limit=100&fib_length=10"
{
  "server": "hostname",
  "prime_limit": 100,
  "fib_length": 10,
  "computation_result": "Computed 25 primes (limit: 100), fibonacci[9] = 34",
  "primes_count": 25,
  "fibonacci_value": 34,
  "execution_time_ms": 546
}
```

**Parameters:**
- `prime_limit`: Integer between 2 and 10,000 (number of primes to calculate up to)
- `fib_length`: Integer between 2 and 100 (length of Fibonacci sequence to generate)