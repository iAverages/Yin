FROM node:16

WORKDIR /app
COPY . /app

RUN yarn install && \
    yarn build

ENV PACKAGE "null"

ENTRYPOINT [ "node", "./scripts/docker-entry.sh" ]