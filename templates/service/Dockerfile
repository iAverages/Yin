FROM node:alpine AS builder
RUN apk add --no-cache libc6-compat
RUN apk update
# Set working directory
WORKDIR /app
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=@yin/base --docker
 
# Add lockfile and package.json's of isolated subworkspace
FROM node:18 AS installer
# RUN apk add --no-cache libc6-compat
# RUN apk update
WORKDIR /app
 
# First install the dependencies (as they change less often)
COPY .gitignore .gitignore
COPY --from=builder /app/out/json/ .
COPY --from=builder /app/out/yarn.lock ./yarn.lock
RUN yarn install
 
# Build the project
COPY --from=builder /app/out/full/ .
COPY turbo.json turbo.json
COPY tsconfig.packages.json tsconfig.packages.json
RUN yarn turbo run build --filter=@yin/base...

FROM node:alpine AS runner
WORKDIR /app/services/base
 
# Don't run production as root
RUN addgroup --system --gid 1001 yin
RUN adduser --system --uid 1001 yin
USER yin
 
COPY --from=installer /app/services/base/package.json /app/services/base/package.json
COPY --from=installer --chown=yin:yin /app/node_modules /app/node_modules
COPY --from=installer --chown=yin:yin /app/services/base/dist /app/services/base/dist
COPY --from=installer --chown=yin:yin /app/services/base/dist /app/services/base/dist

EXPOSE 3000

CMD [ "yarn", "start" ]
