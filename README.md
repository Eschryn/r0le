# r0le
[![Docker Image CI](https://github.com/Eschryn/r0le/actions/workflows/docker-image.yml/badge.svg)](https://github.com/Eschryn/r0le/actions/workflows/docker-image.yml)

## Setting up the development environment 
**Requirements**
* Docker
* cargo + cargo-watch
* buildtools for your system (MSVC, etc)

Set up a redis instance that the bot can use
`docker run --name redis -p 6379:6379 redis`

Watch for changes and run
`cargo watch -x 'r -- -t TOKEN -a APPLICATION_ID'`

## Building for deployment
**Requirements**
* Docker

```
$ docker build . -t r0le:latest
$ docker-compose up -d
```
