##Development container for Intellij Plugin

To build the specific docker container use the provided Dockerfile for the corresponding docker container.

```bash
docker build -t correo_hive -f Dockerfile .
```

Create the correo docker network with the following command.

```bash
docker network create correo
```

To run the docker containers use the provided compose file.

```bash
docker compose up -d
``` 


