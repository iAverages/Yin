# Yin

A Micro Services Discord Bot built with Typescript, MySQL, RabbitMQ.

| Package | Build Status                                                                                                                                                                               |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Gateway | [![Build Status](<https://ci.danielraybone.com/app/rest/builds/buildType:(id:Yin_Gateway)/statusIcon>)](https://ci.danielraybone.com/buildConfiguration/Yin_Gateway?mode=branches&guest=1) |
| Worker  | [![Build Status](<https://ci.danielraybone.com/app/rest/builds/buildType:(id:Yin_Worker)/statusIcon>)](https://ci.danielraybone.com/buildConfiguration/Yin_Worker?mode=branches&guest=1)   |
| Panel   | [![Build Status](<https://ci.danielraybone.com/app/rest/builds/buildType:(id:Yin_Panel)/statusIcon>)](https://ci.danielraybone.com/buildConfiguration/Yin_Panel?mode=branches&guest=1)     |

## Development

This repository contains a `docker-compose.dev.yml` file which can be used to setup a local dev environment containing
all the services needed to run & develop Yin.

```bash
# Starts dev server for Discord bot & web dashboard
yarn run dev

# Start dev server for Discord bot
yarn run dev:bot

# Start dev server for web dashboard (api + panel)
yarn run dev:web
```
