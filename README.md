# ochub-docs

Independent documentation for
[OcHub](https://github.com/OcHub-team/OcHub) and
[ochub-ui](https://github.com/OcHub-team/ochub-ui).

The documentation remains static and searchable, while product screenshots and
step-by-step interface replicas are rendered by a real GPUI application compiled
to WebAssembly. The renderer uses the same `ochub-ui` Rust components and theme
system as the native application; only the sample data is documentation-specific.

The native app's backend services are intentionally not bundled into the browser.
The code editor also remains desktop-only because its Zed text-buffer dependency
currently includes native process and filesystem utilities.

## Development

Requirements: pnpm 11, Rust nightly with `rust-src` and
`wasm32-unknown-unknown`, and [Trunk](https://trunkrs.dev/).

```sh
pnpm install
pnpm dev
pnpm check
```

`pnpm build:wasm` writes generated artifacts to `public/wasm/`; the directory is
ignored and Astro copies it into the final static site.

## Deployment

GitHub Actions compiles the Rust/WASM renderer, builds the static site, runs all
checks, and deploys `dist/` to the `ochub-docs` Cloudflare Worker on successful
pushes to `main`. The workflow can also be run manually from GitHub Actions.

The repository requires `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN`
secrets. Cloudflare Workers Builds must remain disconnected to avoid duplicate
deployments.

Licensed under GPL-3.0-or-later.
