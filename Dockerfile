FROM rust:slim as cargo-build 
RUN apt update && apt install binutils pkg-config -y 
RUN apt install libssl-dev -y  
RUN USER=root cargo new --bin perf 
WORKDIR /perf 

COPY ./Cargo.toml ./Cargo.toml 
COPY ./Cargo.lock ./Cargo.lock 
RUN cargo build --release 

RUN rm -f target/release/deps/perf* && rm src/*.rs  
COPY ./src ./src 
RUN cargo build --release

