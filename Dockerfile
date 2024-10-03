FROM rust:1.76.0 as build

# create a new empty shell project
RUN USER=root cargo new --bin aznxt-ws-server
WORKDIR /aznxt-ws-server

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/aznxt_ws_server*
RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

COPY --from=build /aznxt-ws-server/target/release/aznxt-ws-server .

ARG ZERODHA_API_KEY
ENV ZERODHA_API_KEY=$ZERODHA_API_KEY

ARG ZERODHA_ACCESS_TOKEN
ENV ZERODHA_ACCESS_TOKEN=$ZERODHA_ACCESS_TOKEN

ARG NODE_JS_SERVICE_AUTH_TOKEN
ENV NODE_JS_SERVICE_AUTH_TOKEN=$NODE_JS_SERVICE_AUTH_TOKEN

ARG ORDER_MANAGEMENT_SERVICE_AUTH_TOKEN
ENV ORDER_MANAGEMENT_SERVICE_AUTH_TOKEN=$ORDER_MANAGEMENT_SERVICE_AUTH_TOKEN

ARG JWT_SECRET
ENV JWT_SECRET=$JWT_SECRET

EXPOSE 9000

CMD ["./aznxt-ws-server"]