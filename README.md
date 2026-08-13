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

Licensed under GPL-3.0-or-later.
