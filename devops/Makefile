#TODO
# - docker-compose re-runs build always (it's fast because of cache but anyway)

RELEASE ?= 1
OSNAME ?= xenial

ifeq ($(RELEASE),1)
CARGO_FLAGS = --release
BUILD_TYPE = release
else
BUILD_TYPE = debug
endif

ifndef PROJECT_DIR
PROJECT_DIR := $(shell git rev-parse --show-toplevel)
endif

# project variables
SRC_DIR_NAME := libaurorawallet
SRC_DIR := $(PROJECT_DIR)/$(SRC_DIR_NAME)
SRC_VERSION := $(shell grep -oP '^version = "\K.*(?=")' $(SRC_DIR)/Cargo.toml)

# cargo related variables
CARGO_TARGET_DIR ?= target/$(OSNAME)
CRATE_P_VERSION ?= $(SRC_VERSION)
CARGO_LOGIN_TOKEN ?=

# libs related variables
LIBNAME := $(SRC_DIR_NAME)
LIB_TARGET_DIR = $(SRC_DIR)/$(CARGO_TARGET_DIR)/$(BUILD_TYPE)
LIB_STATIC = $(LIB_TARGET_DIR)/$(LIBNAME).a
LIB_DYNAMIC = $(LIB_TARGET_DIR)/$(LIBNAME).so
LIBS = $(LIB_DYNAMIC) $(LIB_STATIC)

# package related variables
PRE_PACKAGE_GOALS := pre_package $(LIB_DYNAMIC)
FPM_P_MAINTAINER := Evernym
FPM_P_URL := https://github.com/evernym/aurora-wallet
FPM_P_LICENSE := Apache License 2.0
FPM_P_VENDOR := Evernym
FPM_P_DESCRIPTION := indy-sdk plugged wallet for AWS Aurora writen in Rust
FPM_P_NAME = $(LIBNAME)
FPM_P_OUTPUT_DIR = $(LIB_TARGET_DIR)
FPM_ARGS = $(LIB_DYNAMIC)=/usr/lib/

# docker related variables
DOCKER_NAME ?= evernym/aurora-wallet
DOCKER_TAG ?=
EXPORT_ENV += OSNAME RELEASE CARGO_TARGET_DIR CRATE_P_VERSION CARGO_LOGIN_TOKEN

.PHONY: all test_dry test build pre_package clean

all: test_dry test package

include shared/Makefile

export OSNAME

image_ci_version:
	echo $(call docker_img_version,'CI_ENV_VERSION',docker/ci/$(OSNAME)/Dockerfile)

image_ci: image_rust
ifeq ($(DOCKER_TAG),)
	$(eval DOCKER_TAG=$(call docker_img_version,'CI_ENV_VERSION',docker/ci/$(OSNAME)/Dockerfile)-$(OSNAME)-ci)
endif
	OSNAME=$(OSNAME) \
	CI_DOCKER_NAME=$(DOCKER_NAME) \
	CI_DOCKER_TAG=$(DOCKER_TAG) \
		docker-compose -f docker/docker-compose.yml build ci

CI_IMAGE_BASED_TARGETS = test_dry test build $(LIBS) package publish_crate clean

$(patsubst %,image_%, $(CI_IMAGE_BASED_TARGETS)): image_ci

test_dry: $(SRC_DIR)
	echo "Running tests in dry mode (compile, but don't run tests)"
	cd $< && cargo clean && RUST_TEST_THREADS=1 RUST_BACKTRACE=1 cargo test $(CARGO_FLAGS) --no-run

test: $(SRC_DIR)
	echo "Running tests"
	cd $< && cargo clean && RUST_TEST_THREADS=1 RUST_BACKTRACE=1 RUST_LOG=trace cargo test $(CARGO_FLAGS)

build $(LIBS): $(SRC_DIR)
	echo "Compiling the project"
	cd $< && cargo clean && cargo build $(CARGO_FLAGS)

pre_package: $(LIB_DYNAMIC)
	rm -f $(LIB_TARGET_DIR)/$(LIBNAME)*$(SRC_VERSION)*.$(FPM_P_OUTPUT_TYPE)

publish_crate: $(SRC_DIR)
	echo "Publishing crate"
ifneq ($(CARGO_LOGIN_TOKEN),)
	cargo login $(CARGO_LOGIN_TOKEN)
else
	$(warning CARGO_LOGIN_TOKEN is not defined, skip login)
endif
	sed -i 's/^version = ".*"$$/version = "$(CRATE_P_VERSION)"/' $</Cargo.toml
	cd $< && cargo package --allow-dirty
	cd $< && cargo publish --allow-dirty

clean: $(SRC_DIR)
	cd $< && cargo clean
