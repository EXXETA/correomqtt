# CorreoMQTT
A modern graphical MQTT client using the HiveMQ client library.

![CorreoMQTT Connection View](screenshot.png)

## Prerequisites

* Install Java 13 e.g. from here https://www.azul.com/downloads/zulu-community/ (Note: No JavaFX is required, as it is handled via maven)
* Install Jpackage, if you want to build native installers: https://jdk.java.net/jpackage/

## Build app
`mvn clean package`

## Run jars

`java -jar target/shade/correomqtt.jar`

## Build Installers

Installers must be built on their respective platform.

* `./build_mac.sh`
* `./build_win.sh`
* `./build_linux.sh`

## Licence 

Licensed under GPL v3.0.
