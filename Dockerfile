FROM node:16

WORKDIR /app

## We copy these first so that Docker can 
## Cache these layers so we do not need to
## run yarn install for every build
COPY package.json package.json
COPY yarn.lock yarn.lock
RUN yarn install

COPY . /app

RUN yarn build

ENV PACKAGE "null"

ENTRYPOINT [ "bash", "./scripts/docker-entry.sh" ]