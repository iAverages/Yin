FROM node:16

WORKDIR /app
COPY . /app

RUN yarn install && \
    yarn build

ENV PACKAGE "null"

ENTRYPOINT [ "bash", "./scripts/docker-entry.sh" ]