# Base for all stages
FROM node:lts-alpine AS base

WORKDIR /app

# Download dependencies
# and build packages
FROM base AS dependencies

# Global package.json
COPY package.json .
COPY yarn.lock .
COPY tsconfig.json .
COPY tsconfig.packages.json . 

# Packages
COPY packages/common packages/common
COPY packages/gateway packages/gateway

RUN yarn install  --pure-lockfile --non-interactive --cache-folder ./yarncache; rm -rf ./yarncache

WORKDIR /app/packages/common

RUN yarn build 

FROM base

WORKDIR /app/packages/gateway

COPY --from=dependencies /app/ /app/

RUN yarn build

EXPOSE 4000

CMD [ "yarn", "start" ]
