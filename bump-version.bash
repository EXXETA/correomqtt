#!/usr/bin/env bash

if [[ $1 != v* ]] ; then
  echo "Aborting: version must begin with a 'v'"
  exit 1
fi
echo "set version to $1"
mvn versions:set -DnewVersion="$1"
echo -n "$1" > ./src/main/resources/org/correomqtt/business/utils/version.txt
echo version.txt: $(cat ./src/main/resources/org/correomqtt/business/utils/version.txt)
