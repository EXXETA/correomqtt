#!/usr/bin/env bash

cd "$GITHUB_WORKSPACE" || exit 1

echo "==== DECLARE CORREO VERSION ===="
export CORREO_VERSION

echo $GITHUB_REF

if [[ "$GITHUB_REF" =~ [^v[0-9]+\.[0-9]+\.[0-9]] ]]; then
  echo "tag set -> set version to tag version"
  CORREO_VERSION=$(echo "$GITHUB_REF" | cut -d "v" -f 2)
else
  echo "Not a release version. Exit"
  exit 1
fi

echo "CORREO_VERSION is $CORREO_VERSION"

echo "==== INSTALL DEPENDENCIES ===="

if [ ! -d "zulu13.29.9-ca-jdk13.0.2-linux_x64" ]; then
  echo -n "Downloading Java 13 ..."
  curl -s https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-linux_x64.tar.gz --output zulu13.29.9-ca-jdk13.0.2-linux_x64.tar.gz
  echo " done"
  echo -n "Extracting Java 13 ..."
  tar zxvf zulu13.29.9-ca-jdk13.0.2-linux_x64.tar.gz >/dev/null 2>&1
  echo " done"
else
  echo "Skip downloading Java 13, because directory zulu13.29.9-ca-jdk13.0.2-linux_x64 already exists."
fi
export JAVA_HOME=$GITHUB_WORKSPACE/zulu13.29.9-ca-jdk13.0.2-linux_x64
export PATH=$JAVA_HOME/bin:$PATH

echo "JAVA_HOME=$JAVA_HOME"
echo "PATH=$PATH"
echo "JAVA_VERSION"
java -version
echo "MVN_VERSION"
mvn -B -version

echo "==== SET CORREO VERSION ===="
mvn -B versions:set -DnewVersion="$CORREO_VERSION"
echo -n "$CORREO_VERSION" >./src/main/resources/org/correomqtt/business/utils/version.txt

echo "==== BUILD CORREO ===="
mvn -B clean install -DskipTests=true

echo "==== DEPLOY TO MAVEN CENTRAL ===="
if [ "$1" = "linux" ] && [[ "$GITHUB_REF" =~ [^v[0-9]+\.[0-9]+\.[0-9]] ]] && [ ! -n $GITHUB_HEAD_REF ]; then
  echo "$GPG_SECRET_KEYS" | base64 --decode | $GPG_EXECUTABLE --import
  echo "$GPG_OWNERTRUST" | base64 --decode | $GPG_EXECUTABLE --import-ownertrust
  echo "tag set -> deploy release to maven central only on linux"
  mvn -B deploy -P release -DskipTests=true --settings "${GITHUB_WORKSPACE}/.github/mvn-deploy.xml"
else
  echo "no tag set -> no deploy"
fi