#!/bin/bash

./node_modules/concurrently/dist/bin/concurrently.js \
"yarn:common run dev" \
"yarn:gateway run dev" \
"yarn:worker run dev" \
"yarn:api run dev"