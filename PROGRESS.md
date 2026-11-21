# Progress

## US edition

- [x] Apple HomeView (US) monitoring — branch `feat/us-homeview` (PR #1, merged 2026-11-21)
  - [x] New `Home` category (`/shop/buy-home`), US-only family `apple-homeview`
  - [x] 6 US SKUs in `us-source/homeview-skus.json`, generated into `products_en_US.json`
  - [x] Product names from `dimensionMerchModel` (standalone / Wall Mounted), footnotes and zero-width chars stripped
  - [x] US pickup query drops `pl=true` (it made HomeView `ineligible`)
  - [x] `ships-to-store` → out of stock with the pickup date kept
  - [x] Category picker only lists categories the selected region has products for
  - [x] Tests: real-page parse fixture, ships-to-store fixture, catalog counts; JS tests pass
  - [x] Fixed CI findings: `App.tsx` possibly-undefined index (tsc), rustfmt layout in new tests
  - [x] macOS CI green (`pnpm test`, `tsc`, `vite build`, `cargo test`) and US live contract tests 6/6
  - [x] Verified in the running app (Home tab, store picker, monitoring) — test build from release workflow run 44954050246
  - [x] Committed, pushed, PR #1, CI green, merged

## CI

- [x] macOS-only CI: Linux and Windows jobs removed (this edition ships macOS DMG only)
- [x] CI gates reduced to JS tests, tsc, frontend build and `cargo test` (no fmt / clippy gate)
- [x] Live contract tests (`live.rs`, `live_stack.rs`) use the US store instead of China mainland, run on macOS

## Repo ownership

- [ ] Point docs and app to the Henryk07 fork — branch `docs/fork-maintainer`
  - [x] README / README_US / README_CA / README_AE: maintainer, download, Issues and clone links; attribution chain suversal → yuanruoyu515 → Henryk07
  - [x] NOTICE: appended Henryk07 modification notice; earlier notices kept
  - [x] Cargo.toml `repository`, app GitHub button, release link and update message
  - [x] Removed the parent maintainer's Threads button
  - [ ] Repo About homepage → Henryk07 Releases
  - [ ] Publish the first Henryk07 release (v2.0.0-us.1) so the download links have a target
  - [ ] Enable GitHub Issues on the fork (docs point feedback there)

## Release v2.0.0-us.1

- [x] Version 2.0.0 in package.json, tauri.conf.json, Cargo.toml, Cargo.lock
- [x] Release notes `docs/releases/v2.0.0-us.1.md`, CHANGELOG 2.0.0, README download links
- [ ] PR merged, tag `v2.0.0-us.1` pushed, release workflow published the universal DMG
