#!/usr/bin/env bash

echo "set version to $1"
mvn versions:set -DnewVersion="$1"
echo -n "v$1" > ./src/main/resources/org/correomqtt/business/utils/version.txt
echo version.txt: $(cat ./src/main/resources/org/correomqtt/business/utils/version.txt)
