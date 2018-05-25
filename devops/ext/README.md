# DevOps tasks automation

This folder provides makefile based API to help with devops tasks automation. Tasks are defined as make targets and could be run either on host or in docker containers. The folder includes the following files / dirs:
- [Makefile](Makefile) defines pattern rule `%_in_docker` to run any target inside docker container and provides a set of other generally useful targets.
- [docker](docker) folder holds docker related routine


## Docker

This module provides a set of dockerfiles based on ubuntu `xenial` and `centos7`. They have an hierarchy:
- `base` docker image includes generally useful packages and `fpm` as well
- `rust` docker image is based on the `base` image. It adds `rust`. Supported arguments:
  - `RUST_VERSION`: version of the `rust` to install. Default: `1.25.0`

## Makefile

### Requirements

- make
- docker
- docker-compose

### Targets

- `%_in_docker` re-runs make for the matched target in docker `$(DOCKER_NAME):$(DOCKER_TAG)`. Requires target `image_%` to be defined and expects that it builds necessary docker image. Environment variables `DOCKER_NAME` and `DOCKER_TAG` should be defined as well.
- `package` creates a package from the source code using [fpm][a1feb9f1] tool. Could be configured by `FPM_*` environment variables.
- `image_base` builds docker image with generally useful packages and `fpm` installed
- `image_rust` builds docker image with `rust` installed

  [a1feb9f1]: https://github.com/jordansissel/fpm "fpm"

Expects the following targets to be defined in child makefiles:
- `image_%`: should be image for target `%`

### Environment variables

- `OSNAME`: switches OS context, possible values: `xenial`, `centos7`. Default: `xenial`
- `EXPORT_ENV`: list of variables that should be exported. Could be expanded by child dockerfiles. The list is also passed to in-docker make targets environment. Default: `OSNAME PROJECT_NAME SRC_VERSION`
- `PROJECT_DIR`: absolute path of the top level project dir. Default: resolved as `git rev-parse --show-toplevel`
- `SRC_VERSION`: version of the source code. Default: `0.1.0`
- `PRE_PACKAGE_GOALS`: space separated list of targets that should be updated before `package` target execution, i.e. $(PRE_PACKAGE_GOALS) is a prerequisite for the target. Default: not set
- `DOCKER_NAME`: name of the image to use in `%_in_docker` target's recipe. Default: not set
- `DOCKER_TAG`: tag of the image to use in `%_in_docker` target's recipe. Default: not set
- `DOCKER_UID`: `uid` of the user passed to `docker run` command. Default: resolved as `id -u`
- `BASE_DOCKER_VERSION`: impacts the tag of the image built by `image_base` target. The tag is evaluated as: `$(BASE_DOCKER_VERSION)-$(OSNAME)`. Default: value of `BASE_ENV_VERSION` environment variable in accordant dockerfile
- `RUST_DOCKER_VERSION`: impacts the tag of the image built by `image_rust` target. The tag is evaluated as: `$(RUST_DOCKER_VERSION)-$(OSNAME)`. Default: value of `RUST_ENV_VERSION` environment variable in accordant dockerfile

Variables to config packaing using [fpm][a1feb9f1] tool:
  - (please refer to [fpm wiki][3c28cd3e] for more information about the fpm command line options)
  - `FPM_P_NAME` (REQUIRED): value for fpm's `--name` option. Default: `$(SRC_VERSION)`
  - `FPM_P_VERSION`: value for fpm's `--version` option. Default: not set
  - `FPM_P_INPUT_TYPE`: value for fpm's `--input-type` option. Default: `dir`
  - `FPM_P_OUTPUT_TYPE`: value for fpm's `--output-type` option. Default: `deb` if `OSNAME=xenial`, `rpm` if `OSNAME=centos7`, otherwise - not set
  - `FPM_P_OUTPUT_DIR`: value for fpm's `--package` option. Default: not set
  - `FPM_P_MAINTAINER`: value for fpm's `--maintainer` option. Default: not set
  - `FPM_P_URL`: value for fpm's `--url` option. Default: not set
  - `FPM_P_LICENSE`: value for fpm's `--license` option. Default: not set
  - `FPM_P_DESCRIPTION`: value for fpm's `--description` option. Default: not set
  - `FPM_ARGS`: string with any fpm arguments to add to the end of the fpm command line. Default: not set
  - ... (please refer to [fpm.mk](ext/fpm.mk) for more details about fpm related environment variables)

  [3c28cd3e]: https://github.com/jordansissel/fpm/wiki "fpm wiki"

Other variables:
- `docker_img_version`: could be used in `call` functions to resolve the value of some `ENV` instruction in dockerfile. Expects the following call: `$(call docker_img_version,<ENV_V_NAME>,<PATH_TO_DOCKERFILE>)`, where `<ENV_V_NAME>` is key in `ENV` instruction in dockerfile with path `<PATH_TO_DOCKERFILE>`
