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
  mkdir wix
  cd wix || exit 1
  echo "Downloading WIX"
  wget -q --no-check-certificate https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip
  echo " done"
  echo -n "Extracting WIX ..."
  unzip wix311-binaries.zip
  echo " done"
  cd ..
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
echo -n "$CORREO_VERSION" > ./src/main/resources/org/correomqtt/business/utils/version.txt

echo "==== BUILD CORREO ===="
mvn clean install -DskipTests=true

echo "==== DEPLOY TO MAVEN CENTRAL ===="
elif [ "$1" = "linux" ]; then
  if [ -n "$TRAVIS_TAG" ]; then
    gpg --fast-import .travis/gpg.asc
    echo "tag set -> deploy release to maven central only on linux";
    mvn --settings "${TRAVIS_BUILD_DIR}/.travis/mvn-settings.xml" org.codehaus.mojo:versions-maven-plugin:2.1:set -DnewVersion=$CORREO_VERSION 1>/dev/null 2>/dev/null
    mvn deploy -P publish -DskipTests=true --settings "${TRAVIS_BUILD_DIR}/.travis/mvn-settings.xml"
  else
    echo "no tag set -> no deploy";
  fi
fi

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
    --vendor "EXXETA AG" \
    --win-upgrade-uuid "146a4ea7-af22-4e1e-a9ea-7945ce0190fd"
    echo " done"

  check if release and deploy manually to github because deploy in windows not working
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
