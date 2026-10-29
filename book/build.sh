#!/bin/sh
# Builds the guide into target/book and finishes it for search engines (tools/guide_seo.mjs: per-page description,
# canonical URL, og:url, breadcrumb JSON-LD, one h1 per page). Use this rather than a bare mdbook build for anything
# that is published.
set -e
cd "$(dirname "$0")"
MDBOOK="${MDBOOK:-$(command -v mdbook || echo "$HOME/.cargo/bin/mdbook")}"
"$MDBOOK" build
node ../tools/guide_seo.mjs --dir ../target/book
