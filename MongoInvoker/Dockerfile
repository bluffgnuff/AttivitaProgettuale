FROM rust:1.59 as builder2

# 1. Create a new empty shell project
RUN USER=root cargo new --bin GenericFunctionWithFlag
WORKDIR ./GenericFunctionWithFlag

# 2. Copy our manifests
COPY ./GenericFunctionWithFlag/Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Build source code
COPY ./GenericFunctionWithFlag/src ./src
ADD . ./
RUN rm ./target/release/deps/GenericFunctionWithFlag*
RUN cargo build --release

FROM rust:1.59 as builder

# 1. Create a new empty shell project
RUN USER=root cargo new --bin MongoInvoker
WORKDIR ./MongoInvoker

# 2. Copy our manifests
COPY ./MongoInvoker/Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Build source code
COPY ./MongoInvoker/src ./src

ADD . ./

RUN rm ./target/release/deps/MongoInvoker*
RUN cargo build --release

FROM debian:bookworm-slim
ARG APP=/usr/src/app

RUN apt-get update
RUN apt-get install -y ca-certificates tzdata
RUN rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /MongoInvoker/target/release/MongoInvoker ${APP}/
COPY --from=builder2 /GenericFunctionWithFlag/target/release/GenericFunctionWithFlag ${APP}/

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./MongoInvoker"]
