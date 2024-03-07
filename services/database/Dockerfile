FROM node:21.7.0-alpine AS builder
RUN apk add --no-cache libc6-compat
RUN apk update
# Set working directory
WORKDIR /app
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=@yin/database --docker
 
# Add lockfile and package.json's of isolated subworkspace
FROM node:21.7.0-alpine AS installer
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
RUN yarn workspace @yin/db prisma generate
RUN yarn turbo run build --filter=@yin/database...

FROM node:21.7.0-alpine AS runner
WORKDIR /app/services/database
 
# Don't run production as root
RUN addgroup --system --gid 1001 yin
RUN adduser --system --uid 1001 yin
USER yin
 
COPY --from=installer /app/services/database/package.json /app/services/database/package.json
COPY --from=installer /app/node_modules /app/node_modules
COPY --from=installer /app/services/database/dist /app/services/database/dist
COPY --from=installer /app/services/database/dist /app/services/database/dist

EXPOSE 3000

ENV NODE_ENV=production

CMD [ "yarn", "start" ]
