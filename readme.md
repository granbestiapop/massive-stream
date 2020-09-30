# Streams Case

## Build and run mock server
Allows bind on :8080 mock server.
```
cd utils/server
docker build --tag mock .
docker run --rm -p8080:8080 -it mock
```

## Build massive publisher using docker
``` 
docker build --tag massive-rust .
```

## Usage
```
docker run --rm -it --network=host massive-rust:latest /bin/bash
FILE=http://host.docker.internal:8080/stream TARGET=http://host.docker.internal:8080/topic ./target/release/perf
```
