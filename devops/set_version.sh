#!/usr/bin/env bash

PACKAGE_VERSION=$1
SUFFIX=$2
BRANCH=$3

VERSION=PACKAGE_VERSION

if [[ BRANCH != "stable" ]]
then
	VERSION=${VERSION}~${SUFFIX}
fi

echo "Version: "${VERSION}


fpm -s deb -t deb --version ${VERSION} libmysqlstorage_${PACKAGE_VERSION}_amd64.deb