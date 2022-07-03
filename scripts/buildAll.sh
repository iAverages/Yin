#!/bin/bash

# Build common first as this is requried by other packages
yarn run common build

# Build other packages in parallel, makes builds faster
./node_modules/concurrently/dist/bin/concurrently.js \
"yarn:gateway run build" \
"yarn:worker run build" \
"yarn:api run build"