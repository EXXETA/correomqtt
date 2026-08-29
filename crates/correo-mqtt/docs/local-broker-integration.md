# Local Broker Integration Harness

The `correo-mqtt` broker integration tests are ignored by default so ordinary
unit tests do not depend on Docker, Mosquitto, open ports, or generated TLS
fixtures.

Run the opt-in harness with:

```bash
cargo test -p correo-mqtt --test local_broker -- --ignored --nocapture
```

Without the env vars below, the broker tests print the missing prerequisite and
return successfully. The reconnect reporting probe runs with a local TCP fake
and does not need an external broker.

## Mosquitto Broker

These commands start one Mosquitto container with MQTT TCP on `1883` and MQTT
over TLS on `8883`. All certificate material is synthetic, short-lived, and
created under a temporary directory.

```bash
tmp="$(mktemp -d)"
chmod 0755 "$tmp"

cat > "$tmp/server.cnf" <<'EOF'
[req]
distinguished_name = dn
req_extensions = v3_req
prompt = no

[dn]
CN = localhost

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
EOF

openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -keyout "$tmp/ca.key" \
  -out "$tmp/ca.crt" \
  -subj "/CN=CorreoMQTT Synthetic Test CA"

openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -keyout "$tmp/untrusted-ca.key" \
  -out "$tmp/untrusted-ca.crt" \
  -subj "/CN=CorreoMQTT Untrusted Test CA"


openssl req -newkey rsa:2048 -nodes \
  -keyout "$tmp/server.key" \
  -out "$tmp/server.csr" \
  -config "$tmp/server.cnf"

openssl x509 -req -days 1 \
  -in "$tmp/server.csr" \
  -CA "$tmp/ca.crt" \
  -CAkey "$tmp/ca.key" \
  -CAcreateserial \
  -out "$tmp/server.crt" \
  -extensions v3_req \
  -extfile "$tmp/server.cnf"

chmod 0644 "$tmp/server.key" "$tmp/server.crt" "$tmp/ca.crt" "$tmp/untrusted-ca.crt"

cat > "$tmp/mosquitto.conf" <<'EOF'
persistence false
allow_anonymous true
log_type error

listener 1883 0.0.0.0
protocol mqtt

listener 8883 0.0.0.0
protocol mqtt
cafile /mosquitto/certs/ca.crt
certfile /mosquitto/certs/server.crt
keyfile /mosquitto/certs/server.key
require_certificate false
EOF

docker run --rm -d --name correo-mqtt-test-broker \
  -p 1883:1883 \
  -p 8883:8883 \
  -v "$tmp/mosquitto.conf:/mosquitto/config/mosquitto.conf:ro" \
  -v "$tmp:/mosquitto/certs:ro" \
  eclipse-mosquitto:2

export CORREO_MQTT_INTEGRATION_BROKER=1
export CORREO_MQTT_BROKER_HOST=localhost
export CORREO_MQTT_BROKER_PORT=1883
export CORREO_MQTT_TLS_HOST=localhost
export CORREO_MQTT_TLS_BROKER_PORT=8883
export CORREO_MQTT_TLS_CA_PEM="$tmp/ca.crt"
export CORREO_MQTT_TLS_UNTRUSTED_CA_PEM="$tmp/untrusted-ca.crt"


cargo test -p correo-mqtt --test local_broker -- --ignored --nocapture

docker rm -f correo-mqtt-test-broker
rm -rf "$tmp"
```

The tests intentionally do not read broker usernames or passwords from the
environment. Keep the broker anonymous or use only synthetic local credentials
in any future auth-specific extension.

## Coverage

- MQTT 3.1.1 and MQTT 5 connect/disconnect.
- Publish/subscribe loopback for QoS 0, QoS 1, and QoS 2 on the broker path.
- Retained message storage and retained delivery on a fresh subscription.
- TLS connect/disconnect with a synthetic CA bundle and hostname validation.
- TLS rejection for an untrusted CA and a hostname absent from the certificate.
- Reconnect state reporting through a local TCP disconnect probe.

SSH routing uses an isolated fake instead of a real local SSH daemon because a
true SSH service requires OS account and forwarding setup. Run the focused fake
transport check with:

```bash
cargo test -p correo-mqtt ssh_driver_rewrites_endpoint_and_closes_tunnel
```
