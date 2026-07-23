# antixt documentation site

This multi-route site is built with antixt itself. It uses ordinary Rust pages,
a dynamic `[slug]` documentation route, shared `view!` components, escaped
`IntoHtml` values, and an opt-in client island for sidebar search. The
landing-page action row dogfoods antixt's typed utility CSS and the docs include
a dedicated `/docs/typed-css` guide.

Application markup uses autocomplete-friendly Rust utilities rather than
semantic class strings: `u::P_4`, `u::GRID`, and `u::TEXT_MUTED` compile to
`p-4`, `grid`, and `text-muted`. Project-specific tokens live in
`components/theme.rs` and use the same checked `Utility` type.

```sh
antixt check docs
antixt dev docs --port 4174
antixt build docs
```

The landing page ships zero JavaScript. Documentation pages load the embedded
`client/docs-search.js` module because their sidebar opts into that island.
Space Grotesk and IBM Plex Mono are requested from Google Fonts with
`display=swap`; self-hosted binary assets can replace that external request once
antixt's static asset pipeline exists.
