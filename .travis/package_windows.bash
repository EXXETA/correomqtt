#!/usr/bin/env bash

cd "$TRAVIS_BUILD_DIR" || exit 1

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

#check if release and deploy manually to github because deploy in windows not working
echo "$TRAVIS_TAG"
if [ -n "$TRAVIS_TAG" ]; then
  echo "tag set -> deploy release to github";
  cd ./target || exit 1;
  gem install dpl --pre;
  dpl releases --token "$GITHUB_API_KEY" --file_glob --file *.msi;
else
  echo "no tag set";
fi