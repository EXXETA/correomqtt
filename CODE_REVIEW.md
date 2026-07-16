# Konsolidierte Gesamt-Code-Review — CorreoMQTT Rust

**Stand:** 2026-07-13  
**Status:** Kanonische Zusammenführung von `CODE_REVIEW.md` und `FULL_CODE_REVIEW.md`; kein Branch-Diff. Dieses Dokument ist die einzige maßgebliche Review; `FULL_CODE_REVIEW.md` ist auf einen Superseded-Verweis reduziert.  
**Methode:** gezielte statische Source-Review nach Runtime/Protokoll, UI/Accessibility, Multi-OS/Packaging, Storage/Plugins/Scripting sowie Tests/Performance; anschließend Remediation mit `cargo fmt --all`, `cargo check --workspace --all-targets --locked` und `cargo test --workspace --all-targets --locked`. Der vollständige Testlauf ist mit 353 bestandenen und 8 explizit ignorierten Tests belegt; reale Keyring-, lokale Broker- und TLS-Pfade bleiben separat abzunehmen.

## Gesamturteil

**P1-Codepfade umgesetzt; Release-Abnahme noch ausstehend.** Klartext-Secret-Persistenz, nicht-atomare Windows-Replacements, stille MQTT-Overruns, unbeschränkte Runtime-/History-Arbeit, blockierender Plugin-Ingress, mutable Action-Tags, pro-Frame-Snapshot-Kopien und der Plugin-Release-Pfad sind im Source behoben. Releasefähig ist der Stand erst nach den realen P1-Abnahmen für Windows-Fehlerfälle, Dauerlast, Plugin-Cancellation und Tag-Release.

## Bewertungsmaßstab

- **P1 — Release-Blocker:** Secret-Exposure, Datenverlust, stiller Nachrichtenverlust, unbegrenztes Ressourcenwachstum/UI-Starvation, defekte Release-Auslieferung oder Release-Supply-Chain-Risiko.
- **P2 — vor breiterem Rollout oder Protokollerweiterung:** aktuelle Funktions-, Portabilitäts-, UX-, Test- oder Erweiterbarkeitslücke.
- **P3 — Wartbarkeit/Robustheit:** begrenzter unmittelbarer Impact; klare Ownership genügt.

## Prüfkriterien

| Kriterium | Status | Begründung |
| --- | --- | --- |
| Modulare Architektur | Teilweise erfüllt | Klare Crates und UI/Core-Trennung; Core-Commands und öffentliche Plugin-ABI sind MQTT-gekoppelt. |
| Fehlerbehandlung | Teilweise erfüllt | Typisierte Fehler/Redaction vorhanden; MQTT-Overruns sind sichtbar, Save-Actions und blockierender Shutdown bleiben P2. |
| Secret Management | Erfüllt | Built-in-Broker-Passwörter werden über stabile `SecretReference`-Werte migriert und nicht erneut in `config.json` serialisiert. |
| Async-Korrektheit | Teilweise erfüllt | Runtime-Budget, bounded Queues, History-Fairness und abbrechbarer WASM-Ingress sind umgesetzt; Script-Deadline und Lifecycle-Timeouts bleiben P2. |
| Speicher/Performance | Teilweise erfüllt | Runtime-Queues sind begrenzt, Snapshot-Kopien erfolgen nur bei Revisionen und Plugin-Ingress läuft außerhalb des UI-Pfads; Tabellenfilterung bleibt P2. |
| Plattformparität | Teilweise erfüllt | Gemeinsamer atomarer Replace-Pfad ersetzt Pre-Delete; Wayland und reale Keyring-/TLS-Integration bleiben P2 beziehungsweise Release-Abnahme. |
| Reproduzierbare Builds/Tests | Teilweise erfüllt | Release-Assets werden veröffentlicht und Actions sind per SHA gepinnt; Toolchain-Mutation und Integrationsabdeckung bleiben P2. |
| Rust-Praxis 2026 | Teilweise erfüllt | Resolver 2, `--locked`, verbotener Unsafe-Code, bounded Queues, Cancellation und atomare Storage-Pfade sind umgesetzt; P2-Lücken verbleiben. |

# P1 — vor Release beheben

## P1-01 — Built-in-Broker-Passwort wird im Klartext serialisiert

**Evidenz:** `crates/correo-core/src/settings_persistence.rs:24-29, 311-316`; `crates/correo-storage/src/current/config.rs:34-45, 166-181`.

`BuiltInBrokerConfig.password` ist ein serialisierbares `String`-Feld. Der Persistenzpfad übergibt es in `AppConfig`; `ConfigStore::save` schreibt die gesamte Konfiguration nach `config.json`. Das Passwort landet damit in Profilkopien, Migrationen und Dateibackups statt ausschließlich im OS-Keyring.

**Kleinste Abhilfe:** Passwort aus dem serialisierbaren Modell entfernen, nur `SecretReference` persistieren, vorhandene Klartextwerte einmalig in `SecretStore` migrieren und erst danach aus der Konfiguration entfernen.

## P1-02 — Windows-Write-Pfad kann den letzten gültigen Zustand löschen

**Evidenz:** `crates/correo-storage/src/current/config.rs:110-126`; `crates/correo-storage/src/current/history.rs:207-222`; `crates/correo-storage/src/migration/safety.rs:408-423`; `crates/correo-storage/src/current/passwords_file_store.rs:109-117`.

Alle Pfade schreiben eine Temp-Datei, löschen unter Windows vorab das Ziel und benennen danach um. Scheitert das Rename durch Lock, AV, I/O-Fehler oder Prozessabbruch, fehlen Config, History, Migrations-/Rollback-Marker oder der verschlüsselte Secret-Blob. Die frühere Aussage „atomare Persistenz überall“ ist damit falsch.

**Kleinste Abhilfe:** Pre-Delete ersatzlos entfernen. Danach Windows-Replace-Semantik, Rollback-/Backup-Strategie, Datei-/Verzeichnis-Synchronisation und Crash-Recovery mit injizierten Fehlern verifizieren; `rename` allein ist kein Nachweis für einen dauerhaften Commit.

## P1-03 — MQTT-Broadcast-Lag verwirft Ereignisse still

**Evidenz:** `crates/correo-mqtt/src/rumqtt/common.rs:24-26, 104-113`; Konsument `crates/correo-core/src/mqtt/service.rs:376-392`.

Beide Tokio-Broadcast-Kanäle haben nur 256 Slots. `RecvError::Lagged(_)` wird mit `continue` verworfen. Eingehende Nachrichten, Publish-/Subscribe-Ergebnisse und Zustandswechsel können ohne Diagnose oder UI-Hinweis verschwinden.

**Kleinste Abhilfe:** Overrun als explizites App-/Transport-Ereignis sichtbar machen. Loss-kritische Pfade über bounded Backpressure führen; Coalescing nur für ausdrücklich ersetzbaren Zustand erlauben und zählen.

## P1-04 — Dauerhafter Eingang kann UI, Speicher und Plugin-Latenz unbeschränkt belasten

**Evidenz:** `crates/correo-core/src/runtime.rs:47-50, 121-139` (`AppRuntime::with_model`, `pump`); `crates/correo-core/src/runtime/plugins.rs:203-229` (`apply_incoming_hooks`); `crates/correo-core/src/mqtt/service.rs:376-392`; `crates/correo-app/src/app.rs:107-113`.

Runtime und MQTT-Service verwenden unbounded Kanäle. `pump()` drainiert bis zur momentanen Leerheit, bevor die UI weiterzeichnen kann. Zusätzlich führt jeder eingehende MQTT-Event vor Model-/History-Dispatch synchron `apply_incoming_hooks` aus; dieses ruft Plugin-Hooks direkt auf. Unter schneller Ingress-Rate oder langsamen Plugins kann `pump()` nicht zeitnah zurückkehren, UI-Eingaben verhungern und Speicher wachsen.

**Kleinste Abhilfe:** pro Frame ein messbares Event-/Zeitbudget setzen, bei Restarbeit Repaint planen, Queues begrenzen und getrennte Overflow-Policies für Nachrichten, Zustand und Commands definieren. Plugin-Hooks aus dem UI-Hot-Path in einen geordneten, abbrechbaren Workerpfad verlagern; Reihenfolge, Cancellation, Diagnostik und Backpressure vorher als Vertrag definieren.

## P1-05 — Workbench-Coalescing kann alle History-Persistierungen blockieren

**Evidenz:** `crates/correo-core/src/history.rs:75-80, 102-139, 283-334`; gemeinsamer Eingang `crates/correo-core/src/runtime.rs:297-336`.

Nach `ReplaceWorkbench` startet die Koaleszierung je Iteration erneut einen 20-ms-Timeout und beendet sich erst nach Ruhe. Bei Dauertraffic wächst `deferred`, während Publish-/Subscription-History nicht persistiert wird. Der Test deckt nur drei unmittelbare Replacements ohne Interleaving ab.

**Kleinste Abhilfe:** feste Deadline beim Eintritt, nur Restzeit abwarten, pro Connection nur den neuesten Workbench-Snapshot behalten und danach deferred Commands sofort abarbeiten. Dauertraffic plus `RecordPublish` als Regression testen.

## P1-06 — Plugin-Repository zeigt auf nicht veröffentlichte Release-Assets

**Evidenz:** `.github/workflows/rust-build.yml:10-11, 241-255, 277-291`.

Der tag-gesteuerte Job generiert URLs unter GitHub Releases, hat aber `contents: read` und lädt ausschließlich ein kurzlebiges Actions-Artefakt hoch. Er erstellt keinen Release und keine referenzierten ZIP-Assets; Plugin-Installationen folgen daher nicht existierenden URLs.

**Kleinste Abhilfe:** Release plus Assets im selben Job mit minimalem `contents: write` veröffentlichen oder einen tatsächlich permanenten Artefakt-Endpunkt verwenden. Download, Prüfsumme und Installation in einem sauberen Profil Ende-zu-Ende prüfen.

**Umsetzungsstand (2026-07-12):** Der Job erzeugt neue Tag-Releases als Draft und akzeptiert Wiederholungen nur für bereits vorhandene Drafts; ein veröffentlichter Release wird vor dem Asset-Upload abgelehnt. Er lädt ZIPs und Repository-JSON hoch, führt Download-/Installations-Smoketest aus und veröffentlicht danach mit `gh release edit "$GITHUB_REF_NAME" --title "$GITHUB_REF_NAME" --draft=false`. Die statische Workflow-Assertion prüft Draft-Erzeugung, Schutz bestehender veröffentlichter Releases und diese Reihenfolge; der fokussierte Test lief erfolgreich. Der Tag-Release-Smoke bleibt die vor Release auszuführende Ende-zu-Ende-Abnahme.

**Reviewer-Follow-up (2026-07-12):** Der Existing-Draft-Pfad veröffentlicht den Release nach erfolgreichem Upload- und Installations-Smoke explizit mit `--draft=false`. Die statische Assertion fordert diesen Schalter und die Reihenfolge Upload → Smoke → Publish. Verifiziert mit `cargo test -p xtask tag_release_workflow_publishes_generated_repository_assets --locked` (1 bestanden, 0 fehlgeschlagen).

## P1-07 — Release-Workflow führt mutable Action-Tags aus

**Evidenz:** `.github/workflows/rust-build.yml:31, 56, 88, 107, 155, 201, 223, 249, 264, 284`.

`actions/checkout@v7`, `actions/cache@v6` und `actions/upload-artifact@v7` sind mutable Major-Tags. Retagging kann CI-/Release-Code mit Token- und Artefaktzugriff ohne Repository-Diff ändern. SHA-verifizierte Tool-Downloads sind eine positive Teilstärke, ersetzen jedoch keine SHA-Pins für Actions.

**Kleinste Abhilfe:** alle Third-Party-Actions an geprüfte vollständige Commit-SHAs pinnen, lesbare Versionslabels als Kommentar beibehalten und Updates reviewbar automatisieren.

## P1-08 — Vollständiger Snapshot wird im UI pro Frame kopiert

**Evidenz:** `crates/correo-ui/src/shell.rs:101-106`; `crates/correo-app/src/app.rs:107-113`; Payload-Struktur `crates/correo-core/src/types.rs:413-425`.

`CorreoUi::draw` klont den vollständigen Snapshot und reicht danach nur Referenzen auf die Kopie weiter. Da die App spätestens alle 100 ms neu zeichnet, werden große Historys und Payloads auch ohne Änderung wiederholt kopiert.

**Kleinste Abhilfe:** in `draw` ausschließlich `&self.snapshot` verwenden. Erst danach den tatsächlichen Snapshot-Übergabepfad mit großen Payloads messen; `Arc<[u8]>` ist eine mögliche zweite Optimierung, nicht der erste Pflichtschritt.

**Umsetzungsstand (2026-07-12):**

- **P1-01:** Broker-Passwörter werden in den Secret-Store migriert; persistiert wird nur eine stabile `SecretReference`. Fehlgeschlagene Secret-/Config-Schritte behalten beziehungsweise restaurieren den vorherigen Zustand.
- **P1-02:** Config, History, `secrets.enc` und Migrationsmarker verwenden gemeinsam `atomic_write_file::AtomicWriteFile`; ein Ziel-Pre-Delete findet nicht mehr statt.
- **P1-03:** Broadcast-Lag wird als expliziter MQTT-Fehler mit Drop-Anzahl bis in sichtbare Core-Diagnostik weitergereicht.
- **P1-04:** App-Commands und -Events sind bounded; `pump()` verarbeitet höchstens 64 Einträge und meldet verbleibende Command-, Event-, MQTT- und Plugin-Result-Backlogs für sofortiges Repaint. Incoming-Hooks laufen geordnet in einem bounded Worker. Der Produktions-Executor klont die ausgewählte `WasmPlugin`-Instanz unter dem Registry-Lock, gibt den Lock vor dem Dispatch frei und unterbricht aktive Aufrufe über frische Wasmtime-Epoch-Cancellation-Tokens. Drop-/Cancellation-Diagnostik bleibt sichtbar und wird vor globaler Ausgabe redigiert.
- **P1-05:** Workbench-Coalescing verwendet eine feste Deadline, hält pro Connection nur den neuesten Ersatz und lässt Publish-/Subscription-Persistenz unter Dauer-Replacements fortschreiten.
- **P1-06:** Tag-Releases laden Repository und Assets in einen Draft, führen den Installations-Smoke aus und veröffentlichen anschließend auch einen bereits vorhandenen Draft explizit mit `--draft=false`.
- **P1-07:** Alle verwendeten GitHub Actions sind auf vollständige Commit-SHAs gepinnt; Versionskommentare bleiben lesbar.
- **P1-08:** `CorreoUi::draw` arbeitet mit `&self.snapshot`; die Desktop-App überträgt einen neuen vollständigen Snapshot nur bei geänderter Model-Revision statt bei jedem Repaint.

**Verifikation:** `cargo fmt --all`, `cargo check --workspace --all-targets --locked` und `cargo test --workspace --all-targets --locked` erfolgreich. Die Regressionstests für Secret-Migration/Rollback, atomaren Replace, sichtbare MQTT-Lags, Pump-Fairness, abbrechbaren geordneten Plugin-Ingress, History-Fairness und Release-Workflow-Reihenfolge liefen erfolgreich. Reale Windows-Crash-/Lock-Injektion, Dauerlast und Tag-Release bleiben Release-Abnahmen.

# P2 — vor breiterem Rollout oder Kafka/AMQP

## P2-01 — Kafka/AMQP benötigen einen transportneutralen Core- und Plugin-Vertrag

**Evidenz:** `crates/correo-core/src/commands.rs:20, 270, 294, 343`; `crates/correo-core/src/mqtt/types.rs:1-143`; `crates/correo-core/src/mqtt/adapter.rs:16-40`; `crates/correo-core/src/model.rs:144-147`; `crates/correo-core/src/model/mqtt.rs:138 ff.`; `crates/correo-plugins/src/abi.rs:10-21, 25-42, 48-68`; `crates/correo-app/src/plugins.rs:522-569`; `crates/correo-core/src/runtime/plugins.rs:159-187`.

`AppCommand`/`AppEvent` tragen MQTT-Typen. Öffentliche Plugin-DTOs tragen MQTT-Felder wie `client_id`, `subscription_topic`, QoS und `retained`; Host-Actions werden teils direkt als `MqttCommand` weitergeleitet. JSON-DTOs allein machen die ABI nicht protokollneutral.

**Kleinste Abhilfe:** oberhalb der Adapter einen kleinen Port für `ConnectionCommand`, `TransportEvent`, `MessageEnvelope`, Consumer-Auswahl und Delivery-Semantik definieren. Protokollmetadaten namespaced/capability-basiert halten; additive ABI v2 mit getesteter v1-Adaptergrenze liefern. Kein neues allgenerisches Protokoll-Plugin-System bauen.

## P2-02 — Connect/Disconnect/Shutdown können hängen

**Evidenz:** `crates/correo-core/src/mqtt/service.rs` (`ServiceLoop::connect`, `close_existing`, `disconnect`, `shutdown_sessions`); `crates/correo-app/src/app.rs` (`Drop for CorreoDesktopApp`).

Connect und Disconnect haben nicht dieselbe Timeout-/Cancellation-Grenze wie Publish/Subscribe. Desktop-Drop wartet blockierend auf MQTT-Shutdown.

**Kleinste Abhilfe:** gleiche Timeout-/Cancellation-Policy für Connect, Disconnect und Shutdown; kurzer Shutdown-Budget, danach Task abbrechen.

## P2-03 — Script-Import ersetzt Bestände destruktiv

**Evidenz:** `crates/correo-storage/src/current/scripting.rs:140-167`.

`ScriptStore::replace_all` löscht erst den kompletten `scripts/`-Baum und schreibt dann einzeln. Jeder Schreibfehler hinterlässt einen leeren oder partiellen Bestand.

**Kleinste Abhilfe:** validierten Snapshot im Geschwister-Temp-Verzeichnis bauen, dann committen und alten Stand bis Commit-Erfolg behalten.

## P2-04 — `SavePayload` wird trotz Capability still verworfen

**Evidenz:** `plugins/save-manipulator/src/lib.rs:54-63`; `crates/correo-plugins/src/abi.rs:240-278`; `crates/correo-app/src/plugins.rs:766-795`; `crates/correo-core/src/types/plugin_workflow.rs:81-106`.

Die Plugin-ABI transportiert `host_actions`, aber `DetailBytesOutput` übernimmt nur Bytes und Content-Type. Eine gewährte `message_save`-Capability erzeugt weder Dialog noch Fehler.

**Kleinste Abhilfe:** capability-geprüfte Host-Actions bis zu einem zentralen Save-Handler transportieren, dort Bestätigung und Dateinamenvalidierung erzwingen und Hook-bis-Host Ende-zu-Ende testen.

## P2-05 — Script-Host-Sleep umgeht die Sandbox-Deadline

**Evidenz:** `crates/correo-scripting/src/executor.rs:62-119, 195-250`.

Die QuickJS-Deadline wird im Interrupt-Handler geprüft; `HostState::sleep` blockiert dagegen in `thread::sleep`-Schleifen und berücksichtigt nur Cancellation, nicht die Deadline.

**Kleinste Abhilfe:** Deadline in `HostState` führen, Sleep auf Restzeit begrenzen und bei Ablauf abbrechen; langen Sleep regressionssicher testen.

## P2-06 — Persistenztests sind schedulerabhängig

**Evidenz:** `crates/correo-core/src/runtime_tests.rs:84-123, 313-324`; `crates/correo-core/src/mqtt/test_support.rs:177-191`; `crates/correo-core/src/settings_persistence.rs:111-116`.

Tests pollen Dateien und nutzen Wanduhr-Sleeps, obwohl ein Worker-Completion-Event verfügbar ist. Das erzeugt Flakes und schwache Erfolgsbeweise.

**Kleinste Abhilfe:** Completion-Event testbar weiterreichen und mit begrenztem `recv_timeout` warten; Polling und feste Sleeps aus Assertions entfernen.

## P2-07 — Reale Keyring-Backends sind nicht in CI getestet

**Evidenz:** `crates/correo-storage/tests/real_keychain.rs:2-3, 20-53`; `.github/workflows/rust-build.yml:78-79`; `Cargo.toml:49`.

Produktive Apple-, Windows- und Secret-Service-Backends sind in ignorierten Tests; die Standardmatrix führt nur `cargo test --workspace --locked` aus.

**Kleinste Abhilfe:** serialisierten echten Keyring-Job pro OS ergänzen, Secret Service/DBus auf Linux bereitstellen und credential-abhängige Migration getrennt behandeln.

## P2-08 — TLS- und SSH-Transportparität ist unvollständig getestet beziehungsweise implementiert

**Evidenz:** `crates/correo-mqtt/tests/local_broker.rs:33-48`; `crates/correo-mqtt/tests/mosquitto.ci.conf:4-5`; `.github/workflows/rust-build.yml:118-126`; `crates/correo-mqtt/src/transport/tls.rs:17-24, 158-162, 186-188`.

TLS-Broker-Tests sind ignoriert und laufen nur als Linux-Klartextpfad. Zusätzlich sind TLS über SSH-Tunnel und PKCS#12-Keystores aktuell Produkt-/Migrationslücken, keine bereits belegten Sicherheitsfehler.

**Kleinste Abhilfe:** kurzlebige CA/Server-Zertifikate im CI-Broker erzeugen, beide MQTT-Versionen und negative Trust-/Hostname-Fälle testen; PKCS#12 und TLS-over-SSH bewusst unterstützen oder als dauerhaftes Limit dokumentieren.

## P2-09 — Plugin-Highlighting blockiert den Detail-Renderpfad

**Evidenz:** `crates/correo-ui/src/workbench_detail.rs:172-191`; `crates/correo-ui/src/payload_highlight.rs:22-27, 42-49`; `crates/correo-app/src/app.rs:93-96`.

Payload und Plugin-IDs werden im Layouter kopiert und ein Plugin-Highlighter synchron ausgeführt. Große Payloads oder langsame Highlighter blockieren die UI.

**Kleinste Abhilfe:** nach Payload-Hash, Plugin-IDs und Theme cachen; außerhalb des Layout-Callbacks berechnen; bis dahin Plain Text zeigen.

## P2-10 — UI-Feedback ist teilweise unzugänglich oder nicht lokalisiert

**Evidenz:** Accessibility `crates/correo-ui/src/workbench_connection_messages.rs:158-179`; `crates/correo-ui/src/workbench_detail.rs:162-170`; Referenz `crates/correo-style/src/widgets.rs:467-474`. i18n `crates/correo-ui/src/toasts.rs:6-12, 54-91`; `crates/correo-ui/src/i18n.rs:81-118`; weitere sichtbare Literale `crates/correo-ui/src/workbench_connection_messages.rs:97-160, 520-620`.

Icon-only Aktionen haben keinen zuverlässigen assistiven Namen. Toasts routen nach englischem gerendertem Text; außerdem umgehen weitere Hover-/Kontextmenü-Literale die Fluent-Pipeline.

**Kleinste Abhilfe:** beschriftete Widget-Info für Icon-Aktionen registrieren und mit AccessKit testen. Core soll typisierte Feedback-Kategorien liefern; sichtbare UI-Literale schrittweise über Fluent führen, ohne Text als Verhaltensschlüssel zu verwenden.

## P2-11 — Normale Wayland-Sitzungen werden auf X11 gezwungen

**Evidenz:** `crates/correo-app/src/app.rs:39-50`.

Wenn `WAYLAND_DISPLAY` und `DISPLAY` gesetzt sind, was bei Wayland mit XWayland normal ist, erzwingt die App X11; der Opt-out ist keine sichtbare dokumentierte Nutzerwahl.

**Kleinste Abhilfe:** Winit standardmäßig wählen lassen; einen eng reproduzierbaren Fallback nur als dokumentierte persistierte Einstellung anbieten.

## P2-12 — Package-Versionen und WASM-Builds sind unnötig inkonsistent

**Evidenz:** Version `crates/correo-ui/build.rs:25-26, 50-61`; `xtask/src/package.rs:415-431`; `xtask/src/package/metadata.rs:45-49`. Doppelte Builds `xtask/src/package.rs:50-53`; `crates/correo-app/build.rs:92-125`; `xtask/src/plugin_repository.rs:117-130`; `xtask/src/package/plugins.rs:9-20`.

`APP_VERSION` kann aus Git-Tag und Paketmetadaten aus `CARGO_PKG_VERSION` kommen. Außerdem werden alle gebündelten WASM-Plugins im Packaging zweimal gebaut.

**Kleinste Abhilfe:** Release-Version ausschließlich aus `CARGO_PKG_VERSION` ableiten; genau eine autoritative WASM-Build-/Staging-Strecke verwenden.

## P2-13 — Packaging mutiert die Toolchain während des Builds

**Evidenz:** `xtask/src/plugin_repository.rs:117-143, 175-190`; `.github/workflows/rust-build.yml:197-217`.

Bei fehlendem `wasm32-unknown-unknown` führt der Paketpfad `rustup target add` aus. Das macht Builds netzwerk- und Toolchain-State-abhängig.

**Kleinste Abhilfe:** WASM-Target explizit/provenance-gepinnt im Bootstrap und CI installieren; `xtask package` darf nur validieren.

## P2-14 — JSON-Payload-Persistenz bläht Workbench-Saves auf

**Evidenz:** `crates/correo-core/src/types.rs:413-425`; Workbench-Persistenz `crates/correo-core/src/runtime.rs:297-336`.

`MessageRow.payload` ist `Vec<u8>` mit Standard-Serde. In JSON entsteht eine Zahlenfolge statt einer Byte-Codierung; große Payloads vergrößern und verlangsamen Persistenz und Reload. Der genaue Größenfaktor ist payloadabhängig.

**Kleinste Abhilfe:** explizite Base64- oder äquivalente Byte-Serialisierung einführen, dabei bestehende Zahlenarray-Dateien rückwärtskompatibel lesen und Größen-/Parsezeit-Regressionswerte festlegen.

## P2-15 — Keyring-Fallback ohne Secret Service ist nicht benutzerführend

**Evidenz:** `crates/correo-storage/src/current/passwords.rs:358, 362`; `crates/correo-storage/src/current/passwords_file_store.rs:7`.

Der verschlüsselte Datei-Fallback wird über `CORREOMQTT_MASTER_PASSWORD` aktiviert. Nutzer ohne Secret Service erhalten keinen klaren UI-Pfad zum notwendigen Master-Passwort.

**Kleinste Abhilfe:** erklärten UI-Flow für fehlenden OS-Keyring/Master-Passwort bereitstellen und auf unterstützten Linux-Umgebungen testen.

## P2-16 — Tabellenvirtualisierung erfolgt nach vollständiger Filter-/DTO-Erzeugung

**Evidenz:** `crates/correo-ui/src/workbench_connection_messages.rs:202-221, 320-359`; Repaint `crates/correo-app/src/app.rs:107-113`; Obergrenzen `crates/correo-core/src/model/mqtt.rs:10-11`.

`rows()` filtert und materialisiert die gesamte History als `Vec<ConnectionMessageRow>`, bevor `show_rows` nur sichtbare Zeilen zeichnet. Die Eingangsmenge ist jedoch gecappt (`MAX_INCOMING_MESSAGES = 1_000`, `MAX_PUBLISH_HISTORY_ROWS = 500`): pro Repaint werden maximal ~1.500 kurze Strings gematcht und Referenz-Structs alloziert. Das ist Mikrosekundenbereich, kein unbegrenztes Ressourcenwachstum im Sinne der P1-Definition — die teure Pro-Frame-Arbeit an denselben Zeilen ist der Payload-Clone aus P1-08. Deshalb P2, nicht P1.

**Kleinste Abhilfe:** erst P1-08 beheben und messen; falls die Filterung dann noch auffällt, gefilterte IDs/Indizes nach Nachrichten-, Filter-, Subscription- und Diagnose-Revision cachen und DTOs nur für sichtbare Zeilen erzeugen.

**Umsetzungsstand (2026-07-13):**

- **P2-01:** Transportneutrale Commands/Events und Message-/Delivery-DTOs liegen oberhalb des MQTT-v1-Adapters; Plugin-ABI v2 bleibt additiv und die v1-Grenze ist regressionsgesichert.
- **P2-02:** Connect, Disconnect und Shutdown teilen eine begrenzte Operation-Timeout-Policy; Desktop-Shutdown bricht nach seinem Budget ab.
- **P2-03:** Script-Bestände werden vollständig in einem Geschwisterverzeichnis aufgebaut und erst danach atomar ersetzt; der alte Stand bleibt bis zum Commit erhalten.
- **P2-04:** `SavePayload` erreicht capability-geprüft den Host, validiert portable Dateinamen und öffnet den nativen Speicherdialog. Der eigentliche Schreibvorgang läuft außerhalb des egui-Frames und verwendet den atomaren Storage-Writer; Fehler kommen als lokalisierte Diagnose zurück.
- **P2-05:** Host-Sleep ist durch die verbleibende Script-Deadline begrenzt und reagiert weiterhin auf Cancellation.
- **P2-06:** Persistenztests warten begrenzt auf Worker-Completion-Events statt Dateien mit Wanduhr-Sleeps zu pollen.
- **P2-07:** Die CI-Matrix enthält serialisierte reale Keyring-Jobs für macOS, Windows und Linux mit Secret Service/DBus.
- **P2-08:** Der Broker-CI-Pfad deckt TLS für MQTT 3.1.1 und 5 sowie negative Trust-/Hostname-Fälle ab; SSH/TLS- und PKCS#12-Grenzen sind explizit geprüft beziehungsweise dokumentiert.
- **P2-09:** Plugin-Highlighting läuft in einem bounded Worker; die theme-unabhängigen Syntax-Spans werden nach Payload und Plugin-Auswahl gecacht, die Palette wird beim Layout aus dem aktuellen Theme angewendet. Der Renderpfad zeigt bis zum Ergebnis Plain Text.
- **P2-10:** Nachrichtenaktionen, Fensterüberschriften und Plugin-Speicherfehler laufen durch Fluent in allen gebündelten Locales. Icon-only-Nachrichtenaktionen registrieren beschriftete `WidgetInfo`; ein egui-kittest prüft die AccessKit-Namen.
- **P2-11:** Die App erzwingt bei normalen Wayland-Sitzungen kein X11 mehr; Winit wählt das Backend.
- **P2-12:** Paketversion und Plugin-Staging haben je eine autoritative Quelle; gebündelte WASM-Plugins werden nicht doppelt gebaut.
- **P2-13:** Packaging validiert das vorinstallierte `wasm32-unknown-unknown`-Target, mutiert die Toolchain aber nicht; Bootstrap und CI installieren es explizit.
- **P2-14:** Persistierte Payloads verwenden Base64 und lesen bestehende Zahlenarrays rückwärtskompatibel.
- **P2-15:** Linux-Einstellungen erklären den tatsächlich vorhandenen verschlüsselten Datei-Fallback: Master-Passwort nur über die Startumgebung setzen und CorreoMQTT neu starten; der Wert wird weder persistiert noch geloggt.
- **P2-16:** Bewusst kein zusätzlicher Filter-Cache: Nach P1-08 bleibt die Eingangsmenge hart auf 1.500 Zeilen begrenzt, und es liegt kein Messwert für einen verbleibenden Engpass vor. Zusätzlicher Invalidation-State wäre ohne Profiling-Evidenz nicht gerechtfertigt.

# P3 — Wartbarkeit und Robustheit

## P3-01 — Externe Plugin-Änderungen können Build-Cache verfehlen

**Evidenz:** `crates/correo-app/build.rs:78-85, 92-106, 141-190`.

`rerun-if-changed` emittiert workspace-relative Pfade aus dem Build-Script-Package, obwohl Staging vom Workspace-Root auflöst.

**Kleinste Abhilfe:** absolute Workspace-Root-Pfade emittieren und eine externe Plugin-Änderung zwischen zwei Builds testen.

## P3-02 — PBKDF2-Fallback braucht eine versionierte Härtungsentscheidung

**Evidenz:** `crates/correo-storage/src/current/passwords_file_store.rs:15` (`FILE_KDF_ITERATIONS = 100_000`).

Das ist Kryptohygiene für einen Fallback, kein Ersatz für P1-01.

**Kleinste Abhilfe:** Iterationsänderung mit Versions-/Migrations- und Laufzeitprüfung entscheiden; keine ungetestete Zahl blind erhöhen.

## P3-03 — Feste Pixel-Offsets können bei Zoom oder Fonts brechen

**Evidenz:** `crates/correo-ui/src/workbench_connection_messages.rs:389-395`.

Zeilenlayout verwendet feste Offsets statt TextStyle-Höhen.

**Kleinste Abhilfe:** Positionen aus TextStyle-Metriken ableiten und Zoom-Regression ergänzen.

## P3-04 — Dokumentierte Dateigrößenregel und zentrale Dispatch-Komplexität laufen auseinander

**Evidenz:** `crates/correo-core/src/runtime/plugins.rs` (~1008 Zeilen); `crates/correo-app/src/plugins.rs` (~995); `crates/correo-ui/src/connection_plugins.rs` (~931); `crates/correo-core/src/runtime.rs` (~806); `crates/correo-core/src/commands.rs:20-272`.

Die 500-Zeilen-Regel wird mehrfach verletzt; `pump()` und `AppCommand` konzentrieren viele Domänen. Das ist kein Grund für eine neue globale Abstraktion.

**Kleinste Abhilfe:** Regel zuerst realistisch anpassen oder gezielt entlang vorhandener Verantwortlichkeiten splitten. Domänenspezifische Command-Gruppierung nur bei weiterem messbarem Wachstum erwägen.

## P3-05 — Legacy-Root-Ermittlung dupliziert Plattformlogik

**Evidenz:** `crates/correo-app/src/startup.rs:165-181` (`legacy_roots`).

Direkter `HOME`-/`APPDATA`-Zugriff steht neben vorhandener `directories`-Abstraktion und kann Sonderumgebungen wie MSYS/Git-Bash uneindeutig behandeln.

**Kleinste Abhilfe:** vorhandene Abstraktion nutzen oder die beabsichtigte Sonderbehandlung testen und dokumentieren.

**Umsetzungsstand (2026-07-13):**

- **P3-01:** Das Build-Script emittiert für Workspace-Manifeste und externe Plugin-Crates absolute, vom ermittelten Workspace-Root abgeleitete `rerun-if-changed`-Pfade.
- **P3-02:** `secrets.enc` schreibt Formatversion 2 mit explizitem `pbkdf2-hmac-sha256` und 600.000 Iterationen. Dateien ohne Versionsfelder werden weiterhin mit den bisherigen 100.000 Iterationen gelesen; unbekannte Parameter werden abgelehnt. Die Kompatibilitätsprüfung öffnet sowohl ein synthetisches Legacy-Blob als auch die neu geschriebene Version erneut.
- **P3-03:** Höhe und beide Textpositionen der Message-Zeile werden aus den aktiven `Button`-/`Small`-Fontmetriken berechnet; feste `7`-/`28`-Pixel-Baselines entfallen. Die Screenshot-Matrix enthält zusätzlich `correo-workbench-light-1024x768-zoom-150pct.png` mit echtem egui-Zoomfaktor 1,5 bei unverändertem logischem Viewport. Bei der visuellen Abnahme sind Topic/Zeitstempel-Baseline, Preview/Status-Baseline, Zeilenabstand sowie Clipping und Überlappung zu prüfen.
- **P3-04:** Alle Rust-Quelldateien liegen unter 500 Zeilen. Die großen Plugin-, Runtime-, Bootstrap-, Script- und Tabellenbereiche wurden an Top-Level-Verantwortungsgrenzen in eingebundene Geschwisterdateien getrennt; bestehende Modulpfade bleiben explizit erhalten.
- **P3-05:** Aktuelle und Legacy-Datenpfade liegen zentral in `correo-storage::current::paths`; der App-Startup konsumiert diese API. `BaseDirs` liefert die Host-Verzeichnisse, und die Auswahl prüft ausschließlich den zum Zielsystem passenden Windows-, macOS- oder Unix-Legacy-Pfad; gesetzte MSYS-/Git-Bash-Fremdvariablen erzeugen keine plattformfremden Kandidaten.

**Verifikation:** `cargo fmt --all`, `cargo check -p correo-storage --tests --locked`, `cargo check --workspace --all-targets --locked` und `cargo test --workspace --all-targets --locked` erfolgreich. Der vollständige Lauf meldet 353 bestandene und 8 explizit ignorierte Tests. P3-02-KDF-Kompatibilität, P3-05-Plattformauswahl und die P3-03-Screenshot-Matrix einschließlich 150-%-Zoom liefen erfolgreich. Die P3-02-Laufzeitprüfung ist als separater optimierter Release-Test ausgeführt und bestanden; sie beeinflusst den normalen Debug-Testlauf nicht durch hardwareabhängige Zeitgrenzen. Die manuelle Sichtprüfung der erzeugten Zoomaufnahme und die P3-01-Zwei-Build-Cache-Regression bleiben offen. Die Inventur aller versionierten und unversionierten Rust-Quellen meldet keine Datei mit 500 oder mehr Zeilen.

# Sekundäre Produkt-/Architektur-Roadmap

Diese Punkte sind keine aktuellen Release-Blocker:

- **Paketarchitekturen:** unterstützte Matrix bewusst dokumentieren; derzeit fehlen laut Audit Intel-macOS, ARM-Linux und ARM-Windows.
- **Flüchtiger egui-State:** Auto-Scroll/Fokus-/Memory-State bewusst als nicht persistierten UI-State dokumentieren; erst bei realer Test-/Persistenzanforderung migrieren.
- **Kleinere Tabellenkosten:** Filter-Lowercasing und Timestamp-Formatierung erst nach P1-08/P2-16 messen.

# Positive Evidenz

- Crate-Grenzen halten UI von MQTT-Clients und Persistenz fern; der Core ist headless testbar.
- Newtypes/Validierung, `unsafe_code = "forbid"`, typed errors und Redaction sind solide Grundlagen.
- WASM-Runtime lehnt Imports ab, validiert ABI-/Memory-Exporte und begrenzt Speicher, Instanzen, Tabellen, Fuel sowie Request-/Response-Größen: `crates/correo-plugins/src/runtime.rs:233-282, 350-388`.
- Plugin-Lokalpfade weisen leere, absolute und Parent-Pfade ab: `crates/correo-plugins/src/repository.rs:98-131`.
- Diagnostics redigiert Secret-Marker und MQTT/TCP/WebSocket-Userinfo: `crates/correo-diagnostics/src/lib.rs:55-119`.
- Die Basis-Testmatrix und Packaging-Smokes existieren für Ubuntu, macOS und Windows; die Befunde betreffen die ignorierten realen Keyring-/TLS-Pfade, nicht deren Nichtexistenz.
- `ScrollArea::show_rows`, Tastaturnavigation, Escape/Outside-Click, Theme-Synchronisierung, deterministische ZIP-Tests, nFPM-Pfadnormalisierung und `ProjectDirs`-Override hatten keinen weiteren hochsicheren Befund.

# Abnahme vor Release

1. Built-in-Broker-Secret migrieren und beweisen, dass `config.json`, Backups und Migrationsausgaben keinen Klartext enthalten.
2. Windows-Rename-/Crash-/Lock-Fehler für Config, History, Secret-Blob und Migration injizieren; alter oder neuer vollständiger Stand muss überleben.
3. MQTT-Ingress über Kanalgrenzen und unter Dauerlast testen; Overrun muss sichtbar sein und UI-/Speicherbudget einhalten.
4. Workbench-Replacement unter gleichzeitigem `RecordPublish`/Subscription-Traffic ausführen; Persistenzlatenz begrenzen.
5. Eingehende Plugin-Hooks mit langsamer Ausführung testen; UI-Fairness, Reihenfolge und Cancellation prüfen.
6. Tag-Release erzeugen, Repository-Entry herunterladen, Prüfsumme prüfen und Plugin in sauberem Profil installieren; alle Actions per SHA pinnen.
7. Große History/Payloads messen: Framezeit, Allokationen, Workbench-Dateigröße und Reload; Accessibility-Namen über AccessKit prüfen.
8. Reale Keyring- und TLS-Integration auf Ziel-OS testen, inklusive negativer TLS-Trust-/Hostname-Fälle.
9. Vor einem Kafka-Client transportneutralen Port und ABI-v1/v2-Kompatibilitätsfixture beweisen.
