#!/bin/sh
# Runs every ```sheet and ```cpd example of the book through the engine. See check-examples.mjs.
#   book/check-examples.sh            check all chapters
#   book/check-examples.sh --show f.md   also print each result as text
ORIG_PWD="$PWD"; export ORIG_PWD
cd "$(dirname "$0")" && exec node check-examples.mjs "$@"
