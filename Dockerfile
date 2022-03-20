FROM rust:1.59 as builder2


RUN USER=root cargo new --bin GenericFunctionWithFlag
WORKDIR ./GenericFunctionWithFlag
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build  --release
RUN rm src/*.rs

RUN rm ./target/release/deps/GenericFunctionWithFlag*
RUN cargo build --release

ADD . ./

FROM rust:1.59 as builder

RUN USER=root cargo new --bin MySQLInvoker

WORKDIR ./MySQLInvoker
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

RUN rm ./target/release/deps/MySQLInvoker*
RUN cargo build --release

ADD . ./

FROM debian:buster-slim
ARG APP=/usr/src/app

#RUN apt-get update \
#    && apt-get install -y ca-certificates tzdata \
#    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /MySQLInvoker/target/release/MySQLInvoker ${APP}/MySQLInvoker
COPY --from=builder2 /GenericFunctionWithFlag/target/release/GenericFunctionWithFlag ${APP}/../GenericFunctionWithFlag

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./MySQLInvoker"]

## Usage
#     docker build -f Dockerfile -t maitrenode:1.1 .
## Run the image to test
#     docker run -d --name maitrenode -p 8070:8070/tcp -p 8070:8070/udp maitrenode:1.1
## Tag the image
#     docker tag maitrenode:1.1 bluffgnuff/maitrenode:1.1
## Register the image on DockerHub
#     docker push bluffgnuff/maitrenode:1.1
