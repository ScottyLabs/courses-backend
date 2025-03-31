# Server

https://docs.railway.com/guides/axum

## Development

### Setup

This project supports HTTPS connections using TLS certificates. You must use `mkcert` to generate trusted certificates.

1. Install `mkcert` as instructed [here](https://github.com/FiloSottile/mkcert?tab=readme-ov-file#installation).
2. Create and navigate to a `certs` directory in this one:

```sh
mkdir certs
cd certs
```

3. Generate certificates for the localhost and loopback addresses:

```sh
mkcert localhost 127.0.0.1 ::1
```

This will generate `localhost+2.pem` and `localhost+2-key.pem` in `certs`. The paths in the application already point to these files, but they can be customized with the following environment variables:

```sh
TLS_CERT_PATH="path/to/name.pem"
TLS_KEY_PATH="path/to/name-key.pem"
```

### Visualization

The API is documented with [Redoc](https://crates.io/crates/utoipa-redoc), which makes use of our `rustdoc` comments. To view it:

1. Start the server
2. Navigate to `/redoc` in your browser (e.g., https://localhost:3000/redoc)
