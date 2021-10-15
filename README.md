# Echo Server

This is similar to [jmalloc/echo-server](https://github.com/jmalloc/echo-server) except
that it is based on rust instead of go. Besides HTTP GET, POST, and DELETE it also supports
WebSocket (at http://127.0.0.1:9000/.ws) and SSE connections (at http://127.0.0.1:9000/.sse).

Example GET:
```bash
$ curl -X GET http://127.0.0.1:9000
{"source":"127.0.0.1:65421","headers":[["host","127.0.0.1:9000"],["user-agent","curl/7.64.1"],["accept","*/*"]]}
```

You can do a GET, POST, or DELETE at any path:
```bash
$ curl -X POST http://127.0.0.1:9000/some/dummy/path
{"source":"127.0.0.1:65421","headers":[["host","127.0.0.1:9000"],["user-agent","curl/7.64.1"],["accept","*/*"]]}
```

For websocket connections use can use [websocat](https://github.com/vi/websocat) to test:
```bash
$ websocat ws://127.0.0.1:9000/.ws
hello
hello
```

For SSE connections you can test using your browsers JavaScript console:
```javascript
const sse = new EventSource('http://localhost:9000');
sse.onmessage = console.log
```
