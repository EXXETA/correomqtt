#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

PLUGIN_UPDATE_DATE=`date +"%Y-%m-%d"`
PLUGIN_JSON_TEMPLATE="{\"id\": \"PLUGIN_ID\",\"releases\": [{\"PLUGIN_VERSION\": \"PLUGIN_VERSION\",\"date\": \"$PLUGIN_UPDATE_DATE\", \"url\": \"PLUGIN_JAR\"}]}"

mvn versions:set -DnewVersion="$CORREO_VERSION"
echo "$CORREO_VERSION" > ./src/main/resources/com/exxeta/correomqtt/business/utils/version.txt
mvn clean install -DskipTests=true

function build_plugin() {

  local PLUGIN_VERSION=$1 && shift
  local REPO_NAME=$1 && shift
  local PLUGIN_ID=$1 && shift

  git clone https://github.com/EXXETA/"$REPO_NAME".git --branch "v$PLUGIN_VERSION" --single-branch
  cd "$REPO_NAME" || exit 1
  mvn versions:use-dep-version -DdepVersion="$CORREO_VERSION" -Dincludes=com.exxeta:correomqtt
  mvn clean package
  JAR_NAME=$(echo -n "$(ls target | grep "$PLUGIN_VERSION".jar)")
  cp ./target/"$JAR_NAME" "$TRAVIS_BUILD_DIR"/src/main/resources/com/exxeta/correomqtt/plugin/update/
  cd "$TRAVIS_BUILD_DIR" || exit 1
  sed "s/PLUGIN_JAR/$JAR_NAME/g" <<< sed "s/PLUGIN_ID/$PLUGIN_ID/g" <<< sed "s/PLUGIN_VERSION/$PLUGIN_VERSION/g" <<< "$PLUGIN_JSON_TEMPLATE" >> plugins.json
}

echo "[" >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-json-format json-format-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-contains-string-validator contains-string-validator-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-zip zip-manipulator-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-xml-xsd-validator xml-xsd-validator-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-xml-format xml-format-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-save save-manipulator-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-base64 base64-plugin
echo "," >> plugins.json
build_plugin 0.0.1 correomqtt-plugin-advanced-validator advanced-validator-plugin
echo "]" >> plugins.json

mv plugins.json src/main/resources/com/exxeta/correomqtt/plugin/update/

#build again with plugins
mvn clean install -DskipTests=true