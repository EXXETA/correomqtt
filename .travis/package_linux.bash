#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

./jdk-14/bin/jpackage \
  --type deb \
  -d target \
  -i target/shade \
  -n CorreoMQTT \
  --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
  --app-version $CORREO_VERSION \
  --icon ./src/main/deploy/package/Icon.png

./jdk-14/bin/jpackage \
  --type rpm \
  -d target \
  -i target/shade \
  -n CorreoMQTT \
  --main-jar correomqtt-$CORREO_VERSION-runnable.jar \
  --app-version $CORREO_VERSION \
  --icon ./src/main/deploy/package/Icon.png
