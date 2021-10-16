# Echo Server

It's purpose is similar to that of [jmalloc/echo-server](https://github.com/jmalloc/echo-server) except
that it has the following paths defined:
- http://localhost:9000/
  the root returns a simple HTML page with the headers received and some extra details
- http://localhost:9000/echo
  returns the headers etc in JSON format
- ws://localhost:9000/ws
  Provides a websocket connection to an echo server
- http://localhost:9000/sse
  Provides an event source connection to a tick timer sending the time every 5 seconds.


Beyond this it also supports prometheus metrics at http://127.0.0.1:9000/metrics.

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
