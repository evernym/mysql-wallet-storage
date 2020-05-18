#!/bin/bash

if [ -z "$1" ]; then
  echo "Usage: $0 /path/to/cargo.toml"
  exit 1
fi

if [ ! -z "$CI_COMMIT_TAG" ] ; then
  version=$CI_COMMIT_TAG
else
  last_release_tag=$(git tag -l 'v[0-9]*' | sort -rV | head -n 1)
  short_tag=$(git rev-parse --short HEAD)
  version=${last_release_tag}-${short_tag}
fi

version=$(echo "${version}" | sed 's/^v//')
sed -i "s/^version = \".*\"$$/version = \"${version}\"/" $1
echo "Updated $1:"
grep '^version = ' $1
