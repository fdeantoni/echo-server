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
