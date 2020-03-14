#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

echo "==== DECLARE CORREO VERSION ===="
export CORREO_VERSION

if [ -n "$TRAVIS_TAG" ]; then
  echo "tag set -> set version to tag version"
  CORREO_VERSION=$(echo "$TRAVIS_TAG" | cut -d "v" -f 2)
else
  CORREO_VERSION=99.99.99
fi

echo "CORREO_VERSION is $CORREO_VERSION"

echo "==== INSTALL DEPENDENCIES ===="

if [ "$1" = "osx" ]; then
  echo -n "Downloading Java 13 ..."
  wget -q https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz
  echo " done"
  echo -n "Extracting Java 13 ..."
  tar zxvf zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz >/dev/null 2>&1
  echo " done"
  export JAVA_HOME=$TRAVIS_BUILD_DIR/zulu13.29.9-ca-jdk13.0.2-macosx_x64
  export PATH=$JAVA_HOME/bin:$PATH
  echo -n "Downloading Java 14 ..."
  wget -q https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_osx-x64_bin.tar.gz
  echo " done"
  echo -n "Extracting Java 14 ..."
  tar zxvf openjdk-14_osx-x64_bin.tar.gz >/dev/null 2>&1
  echo " done"
elif [ "$1" = "linux" ]; then
  echo -n "Downloading Java 13 ..."
  wget -q https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-linux_x64.tar.gz
  echo " done"
  echo -n "Extracting Java 13 ..."
  tar zxvf zulu13.29.9-ca-jdk13.0.2-linux_x64.tar.gz >/dev/null 2>&1
  echo " done"
  export JAVA_HOME=$TRAVIS_BUILD_DIR/zulu13.29.9-ca-jdk13.0.2-linux_x64
  export PATH=$JAVA_HOME/bin:$PATH
  echo -n "Downloading Java 14 ..."
  wget -q https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_linux-x64_bin.tar.gz
  echo " done"
  echo -n "Extracting Java 14 ..."
  tar zxvf openjdk-14_linux-x64_bin.tar.gz >/dev/null 2>&1
  echo " done"
elif [ "$1" = "windows" ]; then
  echo "Downloading Unzip"
  wget -q http://stahlworks.com/dev/unzip.exe
  echo " done"
  echo -n "Downloading Java 13 ..."
  wget -q --no-check-certificate https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-win_x64.zip
  echo " done"
  echo -n "Extracting Java 13 ..."
  unzip -q zulu13.29.9-ca-jdk13.0.2-win_x64.zip
  echo " done"
  mv zulu13.29.9-ca-jdk13.0.2-win_x64 zulu13
  echo "Downloading Maven"
  wget -q --no-check-certificate https://mirror.dkd.de/apache/maven/maven-3/3.6.3/binaries/apache-maven-3.6.3-bin.zip
  echo " done"
  echo -n "Extracting Maven ..."
  unzip -q apache-maven-3.6.3-bin.zip
  echo " done"
  mv apache-maven-3.6.3 maven
  echo "Downloading WIX"
  wget -q --no-check-certificate https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip
  echo " done"
  echo -n "Extracting WIX ..."
  unzip -q wix311-binaries.zip wix311-binaries
  echo " done"
  mv wix311-binaries wix
  ls -l
  export JAVA_HOME=$TRAVIS_BUILD_DIR/zulu13
  export PATH=$JAVA_HOME/bin:$PATH
  export M2_HOME=$TRAVIS_BUILD_DIR/maven
  export MAVEN_HOME=$TRAVIS_BUILD_DIR/maven
  export PATH=$M2_HOME/bin:$PATH
  export PATH=$TRAVIS_BUILD_DIR/wix:$PATH
  echo -n "Downloading Java 14 ..."
  wget -q https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_windows-x64_bin.zip
  echo " done"
  echo -n "Extracting Java 14 ..."
  unzip -q openjdk-14_windows-x64_bin.zip
  echo " done"
fi

echo "JAVA_HOME=$JAVA_HOME"
echo "PATH=$PATH"
echo "JAVA_VERSION"
java -version
echo "MVN_VERSION"
mvn -version

echo "==== SET CORREO VERSION ===="
mvn versions:set -DnewVersion="$CORREO_VERSION"
echo "$CORREO_VERSION" > ./src/main/resources/com/exxeta/correomqtt/business/utils/version.txt

echo "==== BUILD CORREO ===="
mvn clean install -DskipTests=true

PLUGIN_UPDATE_DATE=`date +"%Y-%m-%d"`
PLUGIN_JSON_TEMPLATE="{\"id\": \"PLUGIN_ID\",\"releases\": [{\"PLUGIN_VERSION\": \"PLUGIN_VERSION\",\"date\": \"$PLUGIN_UPDATE_DATE\", \"url\": \"PLUGIN_JAR\"}]}"

function build_plugin() {
  local PLUGIN_VERSION=$1 && shift
  local REPO_NAME=$1 && shift
  local PLUGIN_ID=$1 && shift

  echo "==== BUILD PLUGIN: $PLUGIN_ID:$PLUGIN_VERSION ===="
  git clone https://github.com/EXXETA/"$REPO_NAME".git --branch "v$PLUGIN_VERSION" --single-branch
  cd "$REPO_NAME" || exit 1
  mvn versions:use-dep-version -DdepVersion="$CORREO_VERSION" -Dincludes=com.exxeta:correomqtt
  mvn clean package
  JAR_NAME=$(echo -n "$(ls target | grep "$PLUGIN_VERSION".jar)")
  cp ./target/"$JAR_NAME" "$TRAVIS_BUILD_DIR"/target/shade/plugins
  cd "$TRAVIS_BUILD_DIR" || exit 1
  echo "$PLUGIN_JSON_TEMPLATE" | sed "s/PLUGIN_JAR/$JAR_NAME/g" | sed "s/PLUGIN_ID/$PLUGIN_ID/g" | sed "s/PLUGIN_VERSION/$PLUGIN_VERSION/g" >> plugins.json
}

mkdir -p target/shade/plugins

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

mv plugins.json mkdir -p target/shade/plugins/

echo "==== PACKAGE CORREO ===="
if [ "$1" = "osx" ]; then
  echo -n "Package DMG ..."
  ./jdk-14.jdk/Contents/Home/bin/jpackage \
    --type dmg \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.icns
    echo " done"
elif [ "$1" = "linux" ]; then
  echo -n "Package DEB ..."
  ./jdk-14/bin/jpackage \
    --type deb \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.png
    echo " done"
    echo -n "Package RPM ..."
  ./jdk-14/bin/jpackage \
    --type rpm \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.png
    echo " done"
elif [ "$1" = "windows" ]; then
  echo -n "Package MSI ..."
  ./jdk-14/bin/jpackage \
    --type msi \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.ico \
    --win-dir-chooser \
    --win-menu \
    --win-menu-group CorreoMqtt \
    --win-shortcut \
    --vendor "EXXETA AG"
    echo " done"

  #check if release and deploy manually to github because deploy in windows not working
  echo "==== DEPLOY ===="
  echo "$TRAVIS_TAG"
  if [ -n "$TRAVIS_TAG" ]; then
    echo "tag set -> deploy release to github";
    cd ./target || exit 1;
    gem install dpl --pre;
    dpl releases --token "$GITHUB_API_KEY" --file_glob --file *.msi;
  else
    echo "no tag set";
  fi
fi
