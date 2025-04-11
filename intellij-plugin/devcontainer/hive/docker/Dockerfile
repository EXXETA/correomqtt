FROM --platform=linux/arm64 hivemq/hivemq4:4.7.1

COPY --chown=hivemq:hivemq config.xml /opt/hivemq/conf/config.xml

ENV HIVEMQ_HOME /opt/hivemq

EXPOSE 1883
EXPOSE 8883
EXPOSE 9399
