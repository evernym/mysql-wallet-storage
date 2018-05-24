SHELL := /bin/bash

# TODO
# 	- fail in case of grep fail for set_docker_img_ver
# 	- more fpm arguments
#	- package-static with libname.a

OSNAME ?= xenial

ifndef PROJECT_DIR
PROJECT_DIR := $(shell git rev-parse --show-toplevel)
endif

PROJECT_NAME ?= $(notdir $(PROJECT_DIR))
SRC_VERSION ?= 0.1.0
PRE_PACKAGE_GOALS ?=

docker_img_version = $(shell grep -oP '^ENV $(1)=\K.*' $(2))

# required for in-docker targets
DOCKER_NAME ?=
DOCKER_TAG ?=

# TODO consider to use GID as well if necessary
ifndef DOCKER_UID
DOCKER_UID := $(shell id -u)
endif

# SELF POSITIONING
ROOT_MK_REL_DIR := $(subst $(PROJECT_DIR),.,$(CURDIR))
ROOT_MK_REL_PATH := $(word 1,$(MAKEFILE_LIST))  # is unlikely to be an absolute one
_THIS_MK_ABS_PATH := $(word $(words $(MAKEFILE_LIST)),$(MAKEFILE_LIST))
#	absolute include/read
ifneq ($(wildcard $(abspath $(_THIS_MK_ABS_PATH))),)
_THIS_MK_ABS_PATH := $(abspath $(_THIS_MK_ABS_PATH))
#	relative include/read
else ifneq ($(wildcard $(CURDIR)/$(_THIS_MK_ABS_PATH)),)
_THIS_MK_ABS_PATH := $(CURDIR)/$(_THIS_MK_ABS_PATH))
else
$(error "Failed to determine abspath to this makefile used as '$(_THIS_MK_ABS_PATH)'")
endif
_THIS_MK_ABS_DIR := $(dir $(_THIS_MK_ABS_PATH))
include $(_THIS_MK_ABS_DIR)/fpm.mk

CP = cp
DOCKER = docker
DOCKER_COMPOSE = docker-compose
FPM = fpm

.PHONY: check_fpm_args package check_dockerfile clean_docker clean_build _default_docker_image_target

# list of conditionally (using '?=') defined variables that
# should be passed to in-docker re-run
EXPORT_ENV += OSNAME PROJECT_NAME SRC_VERSION

export $(EXPORT_ENV)

# PACKAGING ROUTINE
check_fpm_args:
ifeq ($(FPM_P_NAME),)
	$(error Package name is not specified)
else ifeq ($(FPM_P_VERSION),)
	$(error Package version is not specified)
endif

# TODO deps with versions like "package (>= 1.0)"
package: check_fpm_args $(PRE_PACKAGE_GOALS)
	$(FPM) $(FPM_ARGS)

# TODO refactor that once compose provides support for images
# build oreder: https://github.com/docker/compose/issues/5228
image_base:
ifeq ($(BASE_DOCKER_VERSION),)
	$(eval BASE_DOCKER_VERSION=$(call docker_img_version,'BASE_ENV_VERSION',$(_THIS_MK_ABS_DIR)/docker/base/$(OSNAME)/Dockerfile))
endif
	BASE_DOCKER_TAG_PREFIX=$(BASE_DOCKER_VERSION)- \
		docker-compose -f $(_THIS_MK_ABS_DIR)/docker/docker-compose.yml build base

image_rust: image_base
ifeq ($(RUST_DOCKER_VERSION),)
	$(eval RUST_DOCKER_VERSION=$(call docker_img_version,'RUST_ENV_VERSION',$(_THIS_MK_ABS_DIR)/docker/rust/$(OSNAME)/Dockerfile))
endif
	RUST_DOCKER_TAG_PREFIX=$(RUST_DOCKER_VERSION)- \
		docker-compose -f $(_THIS_MK_ABS_DIR)/docker/docker-compose.yml build rust

%_in_docker: image_%
	echo "Re-running goal $* in docker $(DOCKER_NAME):$(DOCKER_TAG)"
	$(DOCKER) run --rm -t -u $(DOCKER_UID) \
		-v $(PROJECT_DIR):/tmp/$(PROJECT_NAME) \
		$(DOCKER_NAME):$(DOCKER_TAG) \
		/bin/bash -c \
			'MAKEFLAGS="$(MAKEFLAGS)" $(foreach var,$(EXPORT_ENV),$(var)="$($(var))") \
			$(MAKE) -C /tmp/$(PROJECT_NAME)/$(ROOT_MK_REL_DIR) -f $(ROOT_MK_REL_PATH) $*'

# CLEAN ROUTINE
# TODO update, consider to use docker-compose down --rmi
#clean_docker:
#	$(DOCKER) images -q $(DOCKER_NS)/$(PROJECT_NAME)-* | xargs -r $(DOCKER) rmi -f
#	-rm -f $(patsubst %,$(BUILD_DIR)/%, $(IMAGES))

#clean_build:
#	rm -rf $(BUILD_DIR)
