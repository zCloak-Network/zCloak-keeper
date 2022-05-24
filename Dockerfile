FROM docker.io/paritytech/ci-linux:production as builder

ARG PROFILE=release
WORKDIR /app

COPY . .

RUN set -eux && cargo build --${PROFILE} 


# MAIN IMAGE FOR PEOPLE TO PULL --- small one#
FROM docker.io/debian:buster-slim
LABEL maintainer="zCloak Network"
LABEL description="zCloak Network provides Zero-Knowledge Proof as a Service for public blockchains."

ARG PROFILE=release
WORKDIR /usr/local/bin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/$PROFILE/zcloak-keeper /usr/local/bin

RUN apt-get -y update && \
    apt-get -y install openssl && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/

USER root
