# Unreact

A static site generation framework for Rust using Handlebars and Scss.

Work in progress...

Submit issue [here](https://github.com/darccyy/unreact/issues/new)

# Usage

For a quick start, check out [Unreact Template](https://github.com/darccyy/unreact-template)

## Production

```ps1
cargo run
```

## Development

Run in development mode with `--dev` or `-d`

```ps1
cargo run -- --dev
```

### Automatically Rebuilding

To automatically rebuild in dev mode, on a file change:

Install `cargo-watch` with `cargo install cargo-watch`

Run:

```ps1
cargo watch 'run -d' -i ./.devbuild
```

### Using a justfile

To create an alias for the command in the previous section:

Install `just` with `cargo install just`

Create `./justfile`, containing:

```js
set shell := ["pwsh.exe", "-c"]

dev:
  cargo watch -x 'run -- --dev' -i .devbuild;
```

Run with `just dev`

## GitHub Pages

Create `./.github/workflows/build.yaml`, containing:

```yaml
name: Build

on:
  # Triggers the workflow on push or pull request events but only for the "main" branch
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]

# ? This might be required ?
# permissions:
#   contents: write

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      # Checks-out your repository under $GITHUB_WORKSPACE, so your job can access it
      - name: Checkout 🛎️
        uses: actions/checkout@v3

      # Run compilation script with Rust
      - name: Build 🔧
        run: |
          cargo run

      # Push changes with plugin
      - name: Deploy 🚀
        uses: JamesIves/github-pages-deploy-action@v4
        with:
          # This must be the build directory
          folder: ./build
```

In the 'Pages' tab in your GitHub repository settings, change 'branch' to `gh-pages`, and click 'Save'
