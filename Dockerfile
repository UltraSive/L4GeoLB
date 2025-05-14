# This tells docker to use the Rust official image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy only the essential files for building the Rust application
COPY Cargo.toml Cargo.lock Makefile ./
COPY src/ ./src/

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/l4lb"]