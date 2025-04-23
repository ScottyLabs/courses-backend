FROM rustlang/rust:nightly-alpine as builder

# Install necessary build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Create a new empty project
WORKDIR /usr/src/collie
COPY . .

# Build the server with release optimizations
RUN cargo build --bin server --release

# Use the latest Alpine image for the final container
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates openssl

WORKDIR /app

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/collie/target/release/server .

# Expose the port the server listens on
EXPOSE 3000

# Set the command to run the server
CMD ["./server"]