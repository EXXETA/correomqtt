#!/usr/bin/env bash

cd "$GITHUB_WORKSPACE" || exit 1

echo "==== DECLARE CORREO VERSION ===="
export CORREO_VERSION

if [[ "$GITHUB_REF" =~ [^v[0-9]+\.[0-9]+\.[0-9]] ]]; then
  echo "tag set -> set version to tag version"
  CORREO_VERSION=$(echo "$GITHUB_REF" | cut -d "v" -f 2)
else
  CORREO_VERSION=`cat $GITHUB_WORKSPACE/src/main/resources/org/correomqtt/business/utils/version.txt`
fi

echo "CORREO_VERSION is $CORREO_VERSION"

echo "==== INSTALL DEPENDENCIES ===="

if [ "$1" = "osx" ]; then
  if [ ! -d "zulu13.29.9-ca-jdk13.0.2-macosx_x64" ]; then
    echo -n "Downloading Java 13 ..."
    curl -s https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz --output zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz
    echo " done"
    echo -n "Extracting Java 13 ..."
    tar zxvf zulu13.29.9-ca-jdk13.0.2-macosx_x64.tar.gz >/dev/null 2>&1
    echo " done"
  else
    echo "Skip downloading Java 13, because directory zulu13.29.9-ca-jdk13.0.2-linux_x64 already exists."
  fi
  if [ ! -d "jdk-14.jdk" ]; then
    echo -n "Downloading Java 14 ..."
    curl -s https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_osx-x64_bin.tar.gz --output openjdk-14_osx-x64_bin.tar.gz
    echo " done"
    echo -n "Extracting Java 14 ..."
    tar zxvf openjdk-14_osx-x64_bin.tar.gz >/dev/null 2>&1
    echo " done"
  else
    echo "Skip downloading Java 14, because directory jdk-14 already exists."
  fi
  export JAVA_HOME=$GITHUB_WORKSPACE/zulu13.29.9-ca-jdk13.0.2-macosx_x64
  export PATH=$JAVA_HOME/bin:$PATH
elif [ "$1" = "linux" ]; then
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
  if [ ! -d "jdk-14" ]; then
    echo -n "Downloading Java 14 ..."
    curl -s https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_linux-x64_bin.tar.gz --output openjdk-14_linux-x64_bin.tar.gz
    echo " done"
    echo -n "Extracting Java 14 ..."
    tar zxvf openjdk-14_linux-x64_bin.tar.gz >/dev/null 2>&1
    echo " done"
  else
    echo "Skip downloading Java 14, because directory jdk-14 already exists."
  fi
elif [ "$1" = "windows" ]; then
  if [ ! -d "zulu13" ]; then
    echo "Downloading Unzip"
    curl -s http://stahlworks.com/dev/unzip.exe --output unzip.exe
    echo " done"
    echo -n "Downloading Java 13 ..."
    curl -s -k https://cdn.azul.com/zulu/bin/zulu13.29.9-ca-jdk13.0.2-win_x64.zip --output zulu13.29.9-ca-jdk13.0.2-win_x64.zip 
    echo " done"
    echo -n "Extracting Java 13 ..."
    unzip -q zulu13.29.9-ca-jdk13.0.2-win_x64.zip
    echo " done"
    mv zulu13.29.9-ca-jdk13.0.2-win_x64 zulu13
  else
    echo "Skip downloading Java 13, because directory zulu13.29.9-ca-jdk13.0.2-linux_x64 already exists."
  fi
  if [ ! -d "maven" ]; then
    echo "Downloading Maven"
    curl -s -k https://mirror.dkd.de/apache/maven/maven-3/3.6.3/binaries/apache-maven-3.6.3-bin.zip --output apache-maven-3.6.3-bin.zip
    echo " done"
    echo -n "Extracting Maven ..."
    unzip -q apache-maven-3.6.3-bin.zip
    echo " done"
    mv apache-maven-3.6.3 maven
  else
    echo "Skip downloading Maven, because directory maven already exists."
  fi
  if [ ! -d "wix" ]; then
    mkdir wix
    cd wix || exit 1
    echo "Downloading WIX"
    curl -s -k https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip --output wix311-binaries.zip
    echo " done"
    echo -n "Extracting WIX ..."
    unzip wix311-binaries.zip
    echo " done"
    cd ..
  else
    echo "Skip downloading WIX, because directory wix already exists."
  fi
  if [ ! -d "jdk-14" ]; then
    echo -n "Downloading Java 14 ..."
    curl -s https://download.java.net/java/GA/jdk14/076bab302c7b4508975440c56f6cc26a/36/GPL/openjdk-14_windows-x64_bin.zip --output openjdk-14_windows-x64_bin.zip
    echo " done"
    echo -n "Extracting Java 14 ..."
    unzip -q openjdk-14_windows-x64_bin.zip
    echo " done"
  else
    echo "Skip downloading Java 14, because jdk-14 already exists."
  fi
  export JAVA_HOME=$GITHUB_WORKSPACE/zulu13
  export PATH=$JAVA_HOME/bin:$PATH
  export M2_HOME=$GITHUB_WORKSPACE/maven
  export MAVEN_HOME=$GITHUB_WORKSPACE/maven
  export PATH=$M2_HOME/bin:$PATH
  export PATH=$GITHUB_WORKSPACE/wix:$PATH
fi

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

echo "==== PACKAGE CORREO ===="
mkdir release
if [ "$1" = "osx" ]; then
  echo -n "Package DMG ..."
  ./jdk-14.jdk/Contents/Home/bin/jpackage \
    --type app-image \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-client-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.icns

  openssl req -subj '/CN=correomqtt.org' -config .github/correo.certconfig -x509 -newkey rsa:4096 -keyout correokey.pem -out correocert.pem -days 365 -nodes
  openssl pkcs12 -passout pass:1234 -export -out correomqtt.p12 -inkey correokey.pem -in correocert.pem
  security create-keychain -p 1234 /tmp/correomqtt-db
  security import correomqtt.p12 -k /tmp/correomqtt-db -P 1234 -T /usr/bin/codesign
  security default-keychain -d user -s /tmp/correomqtt-db
  security unlock-keychain -p 1234 /tmp/correomqtt-db
  security list-keychains -s /tmp/correomqtt-db
  codesign -h -fs correomqtt.org --keychain /tmp/correomqtt-db --force target/CorreoMQTT.app

  ./jdk-14.jdk/Contents/Home/bin/jpackage \
    --type dmg \
    -d target \
    -n CorreoMQTT \
    --app-version $CORREO_VERSION \
    --app-image target/CorreoMQTT.app
  echo " done"
  cp *.dmg release/
elif [ "$1" = "linux" ]; then
  echo -n "Package DEB ..."
  ./jdk-14/bin/jpackage \
    --type deb \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-client-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.png \
    --linux-package-deps libpng16-16 \
    --resource-dir .github/resources/linux
  echo " done"
  cp *.deb release/
  echo -n "Package RPM ..."
  ./jdk-14/bin/jpackage \
    --type rpm \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-client-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.png \
    --resource-dir .github/resources/linux
  echo " done"
  cp *.rpm release/
elif [ "$1" = "windows" ]; then
  echo -n "Package MSI ..."
  ./jdk-14/bin/jpackage \
    --type msi \
    -d target \
    -i target/shade \
    -n CorreoMQTT \
    --main-jar correomqtt-client-$CORREO_VERSION-runnable.jar \
    --app-version $CORREO_VERSION \
    --icon ./src/main/deploy/package/Icon.ico \
    --win-dir-chooser \
    --win-menu \
    --win-menu-group CorreoMqtt \
    --win-shortcut \
    --vendor "EXXETA AG" \
    --win-upgrade-uuid "146a4ea7-af22-4e1e-a9ea-7945ce0190fd"
  echo " done"
  cp *.msi release/
fi
