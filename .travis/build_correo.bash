#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

mvn versions:set -DnewVersion="$CORREO_VERSION"
echo "$CORREO_VERSION" > ./src/main/resources/com/exxeta/correomqtt/business/utils/version.txt
mvn install -DskipTests=true