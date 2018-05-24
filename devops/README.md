# Aurora-wallet devops routine

This folder includes devops related routine and consists of the following parts:
- `Makefile` automates devops tasks like test, package and publish to [crates.io](https://crates.io/) which could be performed either on-host or in-docker
- `docker` folder holds docker related routine
- `aws-codebuild` folder consists of files that describes AWS CodeBuild based CI/CD pipelines
- `ext` folder is a [git-subrepo][d003158e] of shared [library](https://github.com/andkononykhin/aurora-wallet/tree/subrepo/devops/ext) which provides makefile based approach of devops tasks automation. Please check its [README.md](ext/README.md) for more information.

  [d003158e]: https://github.com/ingydotnet/git-subrepo "git-subrepo"

## Docker

Aurora wallet is shipped with dockerfiles for ubuntu [xenial](ci/xenial/Dockerfile) and [centos7](ci/xenial/Dockerfile) which describe images with necessary environment for CI/CD tasks on these OSes.

## CI pipeline

CI pipeline is described by [Jenkinsfile.ci](aws-codebuild/Jenkinsfile.ci). It uses [Jenkins shared library](Jenkins shared library) API to build projects on [AWS CodeBuild](https://aws.amazon.com/codebuild/). CI uses docker containers from `docker/ci` folder to run tests on both ubuntu `xenial` and `centos7`.

CI pipeline stages:
- clone GitHub repository
- upload current HEAD as zip archive to AWS S3 bucket used by CodeBuild project
- launch CodeBuild project using `AwsCodeBuildHelper.build` API. It includes a set of sub-stages:
  - (optional) create/update CodeBuild project
  - (optional) create AWS ECR repository to use by CodeBuild project
  - (optional) build docker image and push it to AWS ECR repository
  - run CodeBuild project to perform cargo testing
  - download logs
- archive logs

## CD pipeline

CD pipeline is described by [Jenkinsfile.cd](aws-codebuild/Jenkinsfile.cd). It uses [Jenkins shared library](Jenkins shared library) API as well.

CD pipeline stages:
- clone GitHub repository
- resolve the following parameters:
  - current source version from [Cargo.toml](../libaurorawallet/Cargo.toml)
  - last revision number among the debian packages with the same source version in [Evernym debian repo](https://repo.corp.evernym.com/deb/dists/evernym-agency-dev-ubuntu/)
- evaluate new deb package version baseing on source version, last revision number and current build number
- upload current HEAD as zip archive to AWS S3 bucket used by CodeBuild project
- launch CodeBuild project using `AwsCodeBuildHelper.build` API. It includes a set of sub-stages:
  - (optional) create/update CodeBuild project
  - run CodeBuild project to perform debian packaging
  - download logs
- archive logs
- upload created debian package to [Evernym debian repo](https://repo.corp.evernym.com/deb/dists/evernym-agency-dev-ubuntu/)

## Makefile

### Requirements

- docker-compose

### Targets
- `test_dry` runs `libaurorawallet` tests in "dry" mode: `cargo test --no-run`
- `test` runs `libaurorawallet` tests: `cargo test`
- `build` runs `cargo build`
- `publish_crate` publishes the code to crates.io performing cargo `login`, `package` and `publish` commands
- `image_ci` builds docker image with necessary env for performing both CI and CD tasks
- `image_ci_version` prints current version of docker image (dockerfile) built by `image_ci` target

Inherited from [ext](ext/Makefile):
- `package` performs `deb` / `rpm` packaging using [fpm][349f7485] tool
- `image_base` builds docker image with generally useful packages and `fpm` installed
- `image_rust` builds docker image with `rust` installed

  [349f7485]: https://github.com/jordansissel/fpm "fpm"

Each target could be run in two ways - with or without `_in_docker` postfix: e.g. `test_in_docker` and `test`. In former case the target is run inside docker container (though it makes sense not for all targets), otherwise current host's environment is used.

### Environment variables

- `PROJECT_DIR`: absolute path of top level project dir. Default: resolved as `git rev-parse --show-toplevel`.
- `RELEASE`: adds `--release` flag to cargo `test` and `build` commands if is set to `1`. Default: `1`
- `OSNAME`: switches os contexts, possible values: `xenial`, `centos7`. Default: `xenial`.
- `CARGO_TARGET_DIR`: sets [CARGO_TARGET_DIR](https://doc.rust-lang.org/cargo/reference/environment-variables.html) variable. Default: `target/$(OSNAME)`.
- `CRATE_P_VERSION`: if set overwrites `version` field of `[package]` section in [Cargo.toml](../libaurorawallet/Cargo.toml) before crate publishing. Default: not set.
- `CARGO_LOGIN_TOKEN`: token to perform `cargo login` during crate publishing. Default: not set.
- `DOCKER_NAME`: name of the image built by `image_ci` target. Default: `evernym/aurora-wallet`.
- `DOCKER_TAG`: tag of the image built by `image_ci` target. Default: `<CI_ENV_VERSION>-$(OSNAME)-ci`, where `CI_ENV_VERSION` is the current version of accordant dockerfile.

Inherited from [ext](ext/Makefile):
- `DOCKER_UID`: `uid` of the user passed to `docker run` command. Default: resolved as `id -u`
- `BASE_DOCKER_VERSION`: impacts the tag of the image built by `image_base` target, tag is formed as: `$(BASE_DOCKER_VERSION)-$(OSNAME)`. Default: value of `BASE_ENV_VERSION` env variable in accordant dockerfile.
- `RUST_DOCKER_VERSION`: impacts the tag of the image built by `image_rust` target, tag is formed as: `$(RUST_DOCKER_VERSION)-$(OSNAME)`. Default: value of `RUST_ENV_VERSION` env variable in accordant dockerfile.
- variables to config packaing using [fpm][349f7485] tool:
  - (please refer to to [fpm wiki][3c28cd3e] more information about fpm command line options)
  - `FPM_P_NAME` (REQUIRED): value for fpm's `--name` option. Default: not set.
  - `FPM_P_VERSION`: value for fpm's `--version` option. Default: not set.
  - `FPM_P_INPUT_TYPE`: value for fpm's `--input-type` option. Default: `dir`
  - `FPM_P_OUTPUT_TYPE`: value for fpm's `--output-type` option. Default: `deb` if `OSNAME=xenial`, `rpm` if `OSNAME=centos7`, otherwise - not set.
  - `FPM_P_OUTPUT_DIR`: value for fpm's `--package` option. Default: not set.
  - `FPM_P_MAINTAINER`: value for fpm's `--maintainer` option. Default: not set.
  - `FPM_P_URL`: value for fpm's `--url` option. Default: not set.
  - `FPM_P_LICENSE`: value for fpm's `--license` option. Default: not set.
  - `FPM_P_DESCRIPTION`: value for fpm's `--description` option. Default: not set.
  - ``: value for fpm's `` option. Default: not set.
  - ``: value for fpm's `` option. Default: not set.
  - `FPM_ARGS`: string with any fpm arguments to end to the end of fpm arguments list. Default: not set.
  - ... (please refer to [fpm.mk](ext/fpm.mk) for more details about related environment variables)

  [3c28cd3e]: https://github.com/jordansissel/fpm/wiki "fpm wiki"