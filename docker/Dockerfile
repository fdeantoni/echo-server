FROM rust:alpine AS echo-build

RUN apk add --no-cache ca-certificates gcc musl-dev

ADD . echo-server
RUN cd echo-server && cargo build --release

FROM alpine:3.14

LABEL com.example.test="echo-server"

RUN apk add --no-cache ca-certificates bind-tools curl wget

RUN addgroup -g 1000 -S echo && adduser -u 1000 -D -S -G echo echo

ENV APP_HOME="/opt/echo"

RUN mkdir -p ${APP_HOME} && chown -R echo:echo ${APP_HOME}
COPY --from=echo-build --chown=echo /echo-server/target/release/echo-server /opt/echo

USER echo
WORKDIR ${APP_HOME}

ENV TZ="Etc/UTC"
ENV RUST_LOG="info,echo_server=info"
ENV HOST_PORT="0.0.0.0:9000"

EXPOSE 9000

STOPSIGNAL SIGINT

CMD ["./echo-server"]
