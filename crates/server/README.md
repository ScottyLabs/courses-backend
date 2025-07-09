# Server

https://docs.railway.com/guides/axum

## Development

To run the server, use the following command:

```sh
RUST_LOG=info cargo run
```

### API Docs

The API is documented with [Swagger](https://crates.io/crates/utoipa-swagger), which makes use of our `rustdoc` comments. To view it:

1. Start the server
2. Navigate to `/swagger` in your browser (e.g., https://localhost:3000/swagger)
