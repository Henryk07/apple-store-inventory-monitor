# Progress

## US edition

- [ ] Apple HomeView (US) monitoring — branch `feat/us-homeview`
  - [x] New `Home` category (`/shop/buy-home`), US-only family `apple-homeview`
  - [x] 6 US SKUs in `us-source/homeview-skus.json`, generated into `products_en_US.json`
  - [x] Product names from `dimensionMerchModel` (standalone / Wall Mounted), footnotes and zero-width chars stripped
  - [x] US pickup query drops `pl=true` (it made HomeView `ineligible`)
  - [x] `ships-to-store` → out of stock with the pickup date kept
  - [x] Category picker only lists categories the selected region has products for
  - [x] Tests: real-page parse fixture, ships-to-store fixture, catalog counts; JS tests pass
  - [x] Fixed CI findings: `App.tsx` possibly-undefined index (tsc), rustfmt layout in new tests
  - [ ] macOS CI green (`pnpm test`, `tsc`, `vite build`, `cargo test`)
  - [ ] Verified in the running app (Home tab, store picker, monitoring)
  - [ ] Committed, pushed, PR, CI green, merged

## CI

- [x] macOS-only CI: Linux and Windows jobs removed (this edition ships macOS DMG only)
- [x] CI gates reduced to JS tests, tsc, frontend build and `cargo test` (no fmt / clippy gate)
- [x] Live contract tests (`live.rs`, `live_stack.rs`) use the US store instead of China mainland, run on macOS
