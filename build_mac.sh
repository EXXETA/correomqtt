#!/bin/bash
mvn compile package
jpackage --package-type dmg  -d target -i target/shade -n CorreoMQTT --main-jar correomqtt-0.9-runnable.jar --app-version 0.9 --icon ./src/main/deploy/package/Icon.icns
