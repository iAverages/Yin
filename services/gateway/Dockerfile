FROM node:alpine AS builder
RUN apk add --no-cache libc6-compat
RUN apk update
# Set working directory
WORKDIR /app
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=@yin/gateway --docker
 
# Add lockfile and package.json's of isolated subworkspace
FROM node:20 AS installer
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
RUN NODE_ENV=production yarn turbo run build --filter=@yin/gateway...

FROM node:alpine AS runner
WORKDIR /app/services/gateway
 
# Don't run production as root
RUN addgroup --system --gid 1001 yin
RUN adduser --system --uid 1001 yin
USER yin
 
COPY --from=installer /app/services/gateway/package.json /app/services/gateway/package.json
COPY --from=installer --chown=yin:yin /app/node_modules /app/node_modules
COPY --from=installer --chown=yin:yin /app/services/gateway/dist /app/services/gateway/dist
COPY --from=installer --chown=yin:yin /app/services/gateway/dist /app/services/gateway/dist

EXPOSE 3000

ENV NODE_ENV=production

CMD [ "yarn", "start" ]
