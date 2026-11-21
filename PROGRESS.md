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
  - [ ] `cargo test --workspace` / `tsc` (no local Rust toolchain or pnpm — run in CI or after install)
  - [ ] Verified in the running app (Home tab, store picker, monitoring)
  - [ ] Committed, pushed, PR, CI green, merged
