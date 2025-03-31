# courses-backend

## Development

### Environment Variables

Copy the `.env.example` file to `.env`. To get a `CANVAS_ACCESS_TOKEN`, navigate to the [Canvas settings page](https://canvas.cmu.edu/profile/settings) and press `+ New Access Token`. The `OIDC_ISSUER_URL` comes from the OAuth2 page on the [Ory dashboard](https://console.ory.sh/projects/).

## Running

In this directory, use `cargo run --bin <name>`, where name is one of `courses | syllabi | server`. Before running the server, make sure to check its [README](./crates/server/README.md).
