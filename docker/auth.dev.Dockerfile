FROM node:20-bookworm

ENV PNPM_HOME=/pnpm
ENV PATH=$PNPM_HOME:$PATH

RUN corepack enable

WORKDIR /workspace

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/auth/package.json apps/auth/package.json

RUN pnpm install --frozen-lockfile

COPY apps/auth apps/auth

CMD ["pnpm", "--filter", "@yin/auth", "dev"]
