# Use the official Rust image as a base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./
COPY self_signed_certs ./self_signed_certs

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Use a compatible Ubuntu image to run the binary (with OpenSSL 3)
FROM ubuntu:22.04

# Install OpenSSL and other necessary libraries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*


# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/https-server /usr/local/bin/https-server

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/self_signed_certs self_signed_certs

ARG APP_PORT=3000
ENV APP_PORT=${APP_PORT}
EXPOSE ${APP_PORT}

# Run the binary
CMD ["https-server"]
