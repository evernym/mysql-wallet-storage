#!/usr/bin/env bash

PACKAGE_VERSION=$1
SUFFIX=$2
BRANCH=$3

echo "Version: "${PACKAGE_VERSION}
echo "Suffix: "${SUFFIX}
echo "Branch: "${BRANCH}

VERSION=${PACKAGE_VERSION}

if [[ $BRANCH != "stable" ]]
then
	VERSION=${VERSION}~${SUFFIX}
fi

echo "Version: "${VERSION}

fpm -s deb -t deb --version ${VERSION} libmysqlstorage_${PACKAGE_VERSION}_amd64.deb

if [[ $BRANCH != "stable" ]]
then
	rm libmysqlstorage_${PACKAGE_VERSION}_amd64.deb
fi
