#!/bin/bash
mvn compile package
jpackage --package-type msi  -d target -i target/shade -n CorreoMQTT --main-jar correomqtt-0.9-runnable.jar --app-version 0.9 --icon ./src/main/deploy/package/Icon.ico --win-dir-chooser --win-menu --win-menu-group CorreoMqtt --win-shortcut --vendor "EXXETA AG"
