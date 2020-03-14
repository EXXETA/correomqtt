#!/usr/bin/env bash

echo "set version to $1"
mvn versions:set -DnewVersion="$1"
echo "$1" > ./src/main/resources/com/exxeta/correomqtt/business/utils/version.txt
echo version.txt: $(cat ./src/main/resources/com/exxeta/correomqtt/business/utils/version.txt)