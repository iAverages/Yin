# Yin [![Build Status](<https://ci.danielraybone.com/app/rest/builds/buildType:(id:Yin_Build)/statusIcon>)](https://ci.danielraybone.com/buildConfiguration/Yin_Build?mode=branches&guest=1)

A Micro Services Discord Bot built with Typescript, MongoDB and Redis.

## Development

This repository contains a `docker-compose.dev.yml` file which can be used to setup a local dev environment containing all the services needed to run & develop Yin.

### Gate Ultility Script

Gate is a ultility script that has been made to make accessing the environment easier.

Run a docker compose command:

```
./gate <any valid docker compose command>
```

Enter the environment

```bash
./gate enter
```

To setup the environment and begin development you can run the following commands

```bash
./gate up -d # Starts all the docker containers in the background
./gate enter
cd /home/node

# Start a package using one of the below

# Starts TS compiler + nodemon for all packages
yarn run dev

# Starts TS compiler + nodemon for specified package
yarn run <package> dev

# Starts nodemon for specified package
yarn run <package> dev:start

# Starts TS compiler for specified package
yarn run <package> dev:build
```
