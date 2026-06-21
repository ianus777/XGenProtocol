// Minimal ambient declaration so `common` type-checks standalone (without the
// app's `vite/client` types). When `common` is compiled as part of an app build,
// this merges by declaration-merging with Vite's own `ImportMetaEnv` (the `DEV`
// member is identical, so no conflict).

interface ImportMetaEnv {
  readonly DEV: boolean;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
