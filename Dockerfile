FROM rust:1.66 as builder
COPY . .
RUN cargo build --release

FROM rust:1.66-slim
RUN apt update && apt install -y iproute2 && rm -rf /var/lib/apt/lists/*
COPY --from=builder target/release/comet-cni target/release/install bin/
WORKDIR /bin
CMD [ "./install" ]