#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

wget --no-check-certificate https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-win_x64.zip
wget http://stahlworks.com/dev/unzip.exe
unzip zulu13.29.9-ca-jdk13.0.2-win_x64.zip
mv zulu13.29.9-ca-jdk13.0.2-win_x64 zulu13
wget --no-check-certificate https://mirror.dkd.de/apache/maven/maven-3/3.6.3/binaries/apache-maven-3.6.3-bin.zip
unzip apache-maven-3.6.3-bin.zip
mv apache-maven-3.6.3 maven
wget --no-check-certificate https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip
unzip wix311-binaries.zip
mv wix311-binaries wix
ls -l
export JAVA_HOME=$TRAVIS_BUILD_DIR/zulu13
echo "JAVA_HOME=$JAVA_HOME"
export PATH=$JAVA_HOME/bin:$PATH
export M2_HOME=$TRAVIS_BUILD_DIR/maven
export MAVEN_HOME=$TRAVIS_BUILD_DIR/maven
export PATH=$M2_HOME/bin:$PATH
export PATH=$TRAVIS_BUILD_DIR/wix:$PATH
echo "PATH=$PATH"
echo "JAVA_VERSION=$(java -version)"
echo "MVN_VERSION=$(mvn -version)"
wget https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_windows-x64_bin.zip
unzip openjdk-14_windows-x64_bin.zip