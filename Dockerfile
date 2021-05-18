FROM rustlang/rust:nightly

WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN mkdir ./src
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > ./src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/app*
	
RUN cargo build --release

COPY ./src ./src

RUN cargo clean --release
RUN cargo build --release
