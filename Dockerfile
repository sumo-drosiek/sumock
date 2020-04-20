FROM rust as build

COPY ./ ./

RUN cargo build --release

RUN mkdir -p /build-out

RUN cp target/release/sumock /build-out/

# Ubuntu 18.04
FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /build-out/sumock /

CMD /sumock