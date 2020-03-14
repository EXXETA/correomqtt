#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

wget -q https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz
tar zxvf zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz >/dev/null 2>&1t
export JAVA_HOME=$TRAVIS_BUILD_DIR/zulu13.29.9-ca-jdk13.0.2-macosx_x64
echo "JAVA_HOME=$JAVA_HOME"
export PATH=$JAVA_HOME/bin:$PATH
echo "PATH=$PATH"
echo "JAVA_VERSION=$(java -version)"
echo "MVN_VERSION=$(mvn -version)"
wget -q https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_osx-x64_bin.tar.gz
tar zxvf openjdk-14_osx-x64_bin.tar.gz >/dev/null 2>&1t