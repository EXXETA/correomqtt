# Branch-Review — release/1.0-hardening-rust gegenüber correomqtt#rust

**Stand:** 2026-07-13
**Basis:** `/Users/svsi/Repos/correomqtt` Branch `rust` (Commit `4f0e68208`)
**Umfang:** 57 Commits + uncommittete Working-Tree-Änderungen; 183 getrackte Dateien geändert (+9.432/−21.798 Zeilen) plus 99 neue, noch **untrackte** Dateien (64 `_part_N.rs`-Splits, 35 nach Verantwortung benannte Split-Dateien).
**Methode:** Eigene statische Review des vollständigen Diffs (inkl. untrackter Dateien), skriptbasierte Symbol-Vollständigkeitsprüfung aller `include!`-Splits, skriptbasierter i18n-Schlüsselabgleich über alle 9 Locales, ein vollständiger Subagenten-Review für CI/xtask/Packaging (inkl. Live-Verifikation der Action-SHA-Pins gegen die GitHub-API) sowie geborgene Teilverifikationen dreier abgebrochener Subagenten-Reviews. Keine Testläufe in dieser Session; `CODE_REVIEW.md` diente als Referenz und bleibt unverändert.

---

## Gesamturteil

**Der Branch ist eine substanzielle, überwiegend disziplinierte Härtung — aber im aktuellen Working-Tree-Zustand nicht auslieferbar.** Die dokumentierten P1-/P2-/P3-Fixes aus `CODE_REVIEW.md` sind real im Code nachweisbar (Details unten), die Löschbilanz ist stark negativ (−12.366 Zeilen netto) und die Sicherheitsentscheidungen (Secret-Migration, atomare Writes, TLS-Design, WASM-Limits) sind sauber. Drei Dinge stehen einem Release im Weg: **(1)** die 99 untrackten Split-Dateien, ohne die der Branch nach einem Commit nicht mehr baut, **(2)** die neu eingeführte harte **1-Sekunden-Deadline für alle Nutzer-Scripts**, die das Scripting-Feature faktisch unbrauchbar macht, und **(3)** drei Release-Pipeline-Lücken (Asset-URLs, Publish-Gating, stiller Installer-Skip).

### Antworten auf die Leitfragen

| Frage | Antwort |
| --- | --- |
| Passen alle Änderungen? | Überwiegend ja. Zwei funktionale Regressionen (B2 Script-Deadline; H1–H3 Release-Pipeline) und der uncommittete Zustand (B1) müssen vor Release behoben werden. |
| Ist nichts zu viel gemacht worden? | Fast nichts. Zwei Ausnahmen: die **mechanischen `_part_N.rs`-`include!`-Splits** (Regel-Compliance statt Strukturverbesserung, M2) und der **ungenutzte Transport-Port** (bewusster Vorab-Vertrag, tolerierbar, aber Drift-Risiko, M4). Der Rest ist bemerkenswert schlank: `update_check.rs` (67 Zeilen, ureq + semver), `atomic_file.rs` (10 Zeilen), Löschungen dominieren 2,3:1. |
| Passt alles für Multi-OS? | Weitgehend ja: atomare Writes plattformneutral, 3 Keyring-Backends + verschlüsselter Datei-Fallback, CI-Matrix mit realen Keyring-Jobs auf allen 3 OS, WiX gepinnt, TLS-CI mit Negativfällen. Lücken: Beta-Runner `ubuntu-26.04` als Pflicht-Gate, floatende `stable`-Toolchain, Signierung/Notarisierung (dokumentiertes 1.0-Non-Goal), Architektur-Matrix (kein Intel-macOS/ARM-Linux/ARM-Windows). |
| Passt alles für Kafka nach 1.0? | Gute Ausgangslage mit einer Auflage: Der neue transportneutrale Vertrag (`transport.rs`) existiert und ist getestet, aber **kein Produktionspfad läuft hindurch** — vor dem ersten Kafka-Code muss mindestens ein echter MQTT-Pfad (z. B. Publish) über den Port geführt werden, sonst driftet der Vertrag. Details unter „Kafka-Bewertung". |

---

## Blocker (vor Release/Commit beheben)

### B1 — 99 untrackte Dateien: Branch bricht bei Commit der getrackten Änderungen

**Evidenz:** `git status --short | grep '^??'` → 99 Dateien; z. B. `xtask/src/plugin_repository.rs` besteht nur noch aus `include!("plugin_repository_part_1.rs"); include!("plugin_repository_part_2.rs");`, beide Part-Dateien sind untracked. Gleiches Muster in `crates/correo-core`, `correo-app`, `correo-ui`, `correo-plugins`, `correo-mqtt`, `correo-style`.

Ein `git add -u` + Commit (ohne die untrackten Dateien) liefert einen Branch, der auf frischem Checkout nicht kompiliert. Das ist kein Code-Fehler, sondern ein Zustandsfehler des Working Trees — aber der gefährlichste im gesamten Review.

**Abhilfe:** Alle 99 Dateien committen (`git add -A`), davor M2 entscheiden (echte Module statt `include!`-Parts), damit die Struktur nicht zweimal angefasst wird.

### B2 — Harte 1-Sekunden-Deadline macht Nutzer-Scripts unbrauchbar

**Evidenz:** `crates/correo-scripting/src/executor.rs:26` (`SCRIPT_DEFAULT_DEADLINE = Duration::from_secs(1)`), `:81` (Deadline-Start), `:217` (Host-Cancellation), `:241` (Sleep-Clamp). Eingeführt in Commit `f2dae1d60`; auf dem Basis-Branch existierte **keine** Script-Deadline. Kein Override in `ScriptExecutionRequest` (`executor.rs:160`). Tests `deadline_cancels_tight_javascript_loop` und `deadline_interrupts_host_sleep` (`tests/runtime_parts/runtime_part_2.rs:84-109`) belegen die Absicht.

Die Deadline-Mechanik selbst ist korrekt und gut gebaut (Interrupt-Handler + HostState + Sleep-Clamp, P2-05 sauber umgesetzt). Der **Wert** ist das Problem: Ein Script, das `connect()` zu einem realen Broker macht, mehrere Publishes absetzt oder `sleep(2000)` aufruft, wird nach 1 Sekunde Wanduhrzeit als „Cancelled" abgebrochen. Das Java-Original erlaubt langlaufende Automations-Scripts; hier ist Scripting de facto auf Sub-Sekunden-Transformationen begrenzt.

**Abhilfe:** Deadline pro Ausführung konfigurierbar machen (Feld in `ScriptExecutionRequest`, UI-seitig ein Timeout mit sinnvollem Default, z. B. 30–60 s) oder — minimal — die Konstante auf einen produkttauglichen Wert heben. Die vorhandene Cancellation-Taste bleibt der Not-Aus. Produktentscheidung erforderlich, kein stiller Fix.

---

## Hoch (vor Release beheben)

### H1 — Plugin-Repository-URLs hart auf EXXETA kodiert, Upload geht ins ausführende Repo

**Evidenz:** `xtask/src/plugin_repository_part_1.rs:143-153` (Basis-URL `https://github.com/EXXETA/correomqtt/releases/download/<tag>`), `.github/workflows/rust-build.yml:344` (Smoke lädt via `gh release download` aus dem **ausführenden** Repo — aktuell der Fork `svensieber/correomqtt`).

Der Smoke-Test validiert nie die URLs, die tatsächlich im publizierten `default-repo.json` stehen. Ein Tag-Release aus dem Fork wird grün, publiziert aber ein Repository-File, dessen URLs ins falsche Repo zeigen.

**Abhilfe:** Basis-URL aus `${{ github.server_url }}/${{ github.repository }}` ableiten und im Smoke jede Repository-Entry-URL gegen die realen Asset-URLs des Releases prüfen.

### H2 — Release-Publish hängt nur am `check`-Job

**Evidenz:** `.github/workflows/rust-build.yml:305-309, 388`. Der Publish-Schritt (`gh release edit --draft=false`) hat `needs: check`; `package`, `keyring-integration` und `mqtt-integration` können fehlschlagen, während der Release trotzdem veröffentlicht wird. Die App-Installer werden zudem nie an den Release angehängt (sie bleiben 14-Tage-Workflow-Artefakte).

**Abhilfe:** `needs: [check, package, keyring-integration, mqtt-integration]` für den Publish; Paket-Artefakte an denselben Release hochladen.

### H3 — Fehlende Installer-Tools werden still übersprungen

**Evidenz:** `xtask/src/package/installers.rs:22-33` (`run_or_skip`): fehlendes `nfpm`/`wix` degradiert DEB/RPM/MSI zu einem `println!`-Skip; der Upload bleibt grün, weil der `*.zip`-Glob `if-no-files-found: error` erfüllt. Eine PATH-/Tool-Regression liefert Releases ohne Installer, ohne dass CI rot wird.

**Abhilfe:** `--require-installers`-Modus für den Package-Smoke (oder Datei-Assertions im Workflow), sodass ein fehlender Plattform-Installer den Job fehlschlagen lässt.

---

## Mittel

### M1 — Plugin-Spezifikationsliste doppelt gepflegt

`crates/correo-app/build.rs` und `xtask/src/plugin_repository_part_1.rs:33-80` definieren je eine eigene Liste der 9 gebündelten Plugins. Ein Plugin, das nur in eine Liste eingetragen wird, erscheint lokal, aber nie im Release-Repository (oder umgekehrt) — still. **Abhilfe:** eine Quelle (gemeinsames Include oder Manifest-Scan von `plugins/`).

### M2 — Mechanische `include!`-Splits statt echter Module („zu viel gemacht")

64 der 99 neuen Dateien heißen `*_part_1.rs`/`*_part_2.rs` und werden per `include!` in den alten Dateipfad eingeklebt (z. B. `runtime.rs` → 3 Parts, `model/connections.rs` → 3 Parts, `abi.rs` → 2 Parts). Das erfüllt die 500-Zeilen-Regel aus `AGENTS.md` formal (größte Datei jetzt 495 Zeilen), widerspricht aber der in `CODE_REVIEW.md` P3-04 dokumentierten Absicht „Split entlang Verantwortungsgrenzen": `_part_N` ist ein Schnitt an der Zeilennummer, kein Schnitt an der Verantwortung. `include!`-Parts sind keine Module (kein eigener Scope, geteilte Imports, schlechtere IDE-/rust-analyzer-Unterstützung, überraschend für jeden neuen Leser) — die Navigierbarkeit ist teils schlechter als vorher in einer großen Datei. Die 35 nach Verantwortung benannten Splits (`plugins_incoming.rs`, `bootstrap_startup.rs`, `plugins_abi_install.rs`, …) zeigen, dass es richtig geht.

**Positiv verifiziert:** Ein Symbol-Vollständigkeitsabgleich (HEAD vs. Working Tree) über alle 37 include-Stub-Dateien fand **keinen verlorenen Code**; die 4 fehlenden Symbole (`hook_output`, `storage_built_in_broker`, `ensure_wasm_target_available_or_install`, `install_wasm_target`) sind absichtliche Löschungen mit 0 verbleibenden Referenzen.

**Abhilfe:** Vor dem Commit der Splits entweder (a) die `_part_N`-Dateien in benannte echte Module umwandeln (`mod xyz;` + `pub use`), oder (b) die 500-Zeilen-Regel in `AGENTS.md` auf „Richtwert, harte Grenze 800" lockern und die mechanischen Splits zurücknehmen. Variante (b) ist der kleinere Diff.

**Entscheidung (2026-07-13):** Variante (a) wird nicht als mechanischer Workspace-Umbau umgesetzt. Die compile-sicheren Splits bleiben wegen der verbindlichen 500-Zeilen-Grenze bestehen; alle 99 zuvor unversionierten Rust-Quelldateien sind im Index erfasst. Verantwortungsbasierte Module entstehen nur bei fachlichen Änderungen am jeweiligen Bereich.

### M3 — Floatende Toolchain

`rust-toolchain.toml:2` (`channel = "stable"`): Ein Rust-Point-Release ändert, womit ein Release-Tag gebaut wird. P2-13 ist damit nur halb geschlossen (die xtask-seitige `rustup target add`-Mutation ist real entfernt und durch einen harten `MissingRustTarget`-Fehler ersetzt — verifiziert). **Abhilfe:** konkrete Version pinnen (z. B. `channel = "1.89"`).

### M4 — Transport-Port ist ein unbenutzter Vorab-Vertrag

`crates/correo-core/src/transport.rs` (`MessageEnvelope`, `ConnectionCommand`, `TransportEvent`, `TransportPort`) und `mqtt/v1_adapter.rs`: `from_mqtt_command`/`from_mqtt_event` werden **ausschließlich von den eigenen Tests** aufgerufen; `TransportPort` hat keine Implementierung. Nur `MessageEnvelope` ist produktiv verdrahtet (Plugin-ABI v2, `runtime/plugin_helpers.rs:98`, `correo-app/src/plugins_abi_install.rs:246`). Als bewusster, kleiner, getesteter Kafka-Vorvertrag (P2-01) vertretbar — aber ohne produktiven Konsumenten ist es Scaffolding, das bei jeder MQTT-Änderung unbemerkt veralten kann. **Abhilfe:** siehe Kafka-Bewertung; alternativ den ungenutzten Teil bis zum Kafka-Start entfernen (er ist in einer Stunde wiederhergestellt und läge dann nicht 12 Monate unbewacht herum).

**Entscheidung (2026-07-13):** Die produktive Adoption bleibt bis zum Kafka-Start zurückgestellt. Der aktuelle Vertrag enthält absichtlich keine MQTT-spezifischen Verbindungsoptionen; eine sofortige Verdrahtung würde entweder den neutralen Vertrag mit MQTT-Details verunreinigen oder nur einen unvollständigen Publish-Adapter vortäuschen. Vertrag und Regressionstests bleiben unverändert.

### M5 — Workspace-Version 0.1.0 auf einem 1.0-Release-Branch

`Cargo.toml:26`; nichts prüft `$GITHUB_REF_NAME` gegen `CARGO_PKG_VERSION` — ein `v1.0.0`-Tag publiziert Artefakte namens `CorreoMQTT-0.1.0-…`. **Abhilfe:** Einzeilige Tag-vs.-Version-Assertion in den tag-gesteuerten Jobs.

### M6 — Beta-Runner als Pflicht-Gate

`.github/workflows/rust-build.yml:28`: `ubuntu-26.04` ist ein Beta-Image für den `check`-Job, an dem alles hängt; Image-Churn kann die gesamte Pipeline blockieren (`macos-26`, `windows-2025` sind GA). **Abhilfe:** auf `ubuntu-24.04` bzw. `ubuntu-latest` für das Gate wechseln.

---

## Niedrig

- **L1 — Mutable Docker-Tag:** `eclipse-mosquitto:2` (`rust-build.yml:157`) floatet; für Konsistenz mit der SHA-Pin-Policy Digest pinnen (nur Test-Oberfläche).
- **L2 — xtask-interne Duplikate:** Atomic-Write-Trio dreifach kopiert (`package.rs:227-242`, `checksums.rs:54-71`, `plugin_repository_part_1.rs`), zwei identische rekursive Verzeichniskopien, `write_checksum_files` ist Alias von `write_sidecar`; die Windows-Variante von `replace_file` in xtask macht weiterhin delete-then-rename (das in Storage per P1-02 verbannte Muster — hier harmlos, da nur frische Staging-Dateien, aber inkonsistent).
- **L3 — Cache-Restore-Keys zu breit:** Package-Job (24.04) restauriert `-check-`-Caches von 26.04 (`rust-build.yml:63-64, 271-273`); bläht Caches, kann Build-Script-Outputs gegen fremde System-Libs stale machen. Pro Job-Flavor scopen.
- **L4 — CI-Kosten:** `keyring-integration` ohne Cargo-Cache (Cold Build unter 15-min-Timeout); `push: branches: ["**"]` + `pull_request` doppelt jede PR; `concurrency` mit cancel-in-progress fehlt.
- **L5 — Draft-Reruns hinterlassen Orphan-Assets:** `--clobber` (`rust-build.yml:374-380`) überschreibt nur namensgleiche Assets; nach einem Plugin-Versionsbump bleiben alte ZIPs am Draft, der Smoke ignoriert sie, Publish exponiert sie.
- **L6 — 111 definierte, aber im Code nicht statisch referenzierte i18n-Schlüssel** (mögliche dynamische Konstruktion — prüfen, sonst löschen). Positiv: Alle statisch referenzierten Schlüssel existieren in **allen 9 Locales** (skriptgeprüft, 0 fehlend).
- **L7 — Dependency-Gewicht des Built-in-Brokers:** `rumqttd` zieht axum/hyper/prometheus/clap in eine Desktop-App (Haupttreiber der +839 Zeilen `Cargo.lock`); Feature-Trim prüfen.
- **L8 — Version-Sniffing-Proxy des Built-in-Brokers:** `crates/correo-app/src/builtin_broker.rs:146-185` liest den CONNECT-Prefix ohne Read-Timeout; ein Client, der verbindet und nichts sendet, hält einen Proxy-Task. Loopback-only, daher niedrig — Timeout wäre eine Zeile.
- **L9 — `secrets.enc` ohne explizite 0600-Permissions:** Inhalt ist AES-256-GCM-verschlüsselt, daher niedrig; `OpenOptions.mode(0o600)` unter Unix wäre Hygiene.

---

## Verifikation der CODE_REVIEW.md-Behauptungen

Stichprobenartige, unabhängige Nachprüfung der als „umgesetzt" dokumentierten Punkte:

| Punkt | Status | Evidenz dieser Review |
| --- | --- | --- |
| P1-01 Klartext-Broker-Passwort | ✔ bestätigt | SecretReference-Migration nachvollzogen (Subagent, vor Abbruch bestätigt) |
| P1-02 Atomare Writes | ✔ bestätigt | `current/atomic_file.rs` (10 Zeilen, `atomic-write-file`), von Config/History/Secrets/Markern genutzt; Rest-Inkonsistenz nur in xtask (L2) |
| P1-03 Broadcast-Lag sichtbar | ✔ bestätigt | `rumqtt/common.rs:108-129`: `Lagged(n)` → `MqttError::protocol` mit Drop-Zahl im Stream |
| P1-04 Bounded Pump | ✔ bestätigt | `runtime_part_1.rs:27,147-260`: `PUMP_BUDGET=64`, Command-Reservierung, Backlog→Repaint (`app.rs:105-108`) |
| P1-05 History-Coalescing-Deadline | ✔ bestätigt | Subagent (vor Abbruch verifiziert: „P1-05 verified sound") |
| P1-06 Draft→Upload→Smoke→Publish | ◐ teilweise | Ablauf vorhanden + statischer Ordnungstest; **aber** H1 (URLs) und H2 (Gating) offen |
| P1-07 Action-SHA-Pins | ✔ bestätigt | Live gegen GitHub-API geprüft: checkout=v7.0.0, cache=v6.1.0, upload-artifact=v7.0.1 — SHAs und Kommentare stimmen |
| P1-08 Kein Snapshot-Clone pro Frame | ✔ bestätigt | `shell.rs:111` (`let snapshot = &self.snapshot`) |
| P2-01 Transportneutraler Port | ◐ teilweise | Typen + Tests existieren; produktiv nur `MessageEnvelope` (M4) |
| P2-05 Sleep unter Deadline | ✔ Mechanik / ✘ Wert | Clamp korrekt (`executor.rs:241`); 1-s-Deadline ist B2 |
| P2-07 Reale Keyring-CI | ✔ bestätigt | Per-OS-Jobs inkl. gnome-keyring unter dbus-run-session |
| P2-08 TLS-CI mit Negativfällen | ✔ bestätigt | SAN-Zertifikate + untrusted-CA-Material im Workflow |
| P2-09 Highlight-Worker + Cache | ✔ bestätigt | `payload_highlight.rs`: bounded Cache/Worker, Palette beim Layout |
| P2-11 Kein X11-Zwang | ✔ bestätigt | Kein `WINIT_UNIX_BACKEND`/X11-Forcing mehr in `correo-app` |
| P2-12 Eine WASM-Staging-Strecke | ✔ bestätigt | xtask kopiert neben dem Binary, `MissingArtifact` bei Fehlen; **aber** M1 (Spec-Liste doppelt) |
| P2-13 Keine Toolchain-Mutation | ✔ bestätigt | `rustup target add` entfernt, harter Fehler; Rest: M3 |
| P2-14 Base64-Payloads rückwärtskompatibel | ✔ bestätigt | `types/payload_bytes.rs`: untagged enum liest Legacy-Arrays, schreibt Base64, mit Test |
| P3-01 Absolute rerun-if-changed | ✔ bestätigt | `build.rs:78-96` nutzt `workspace_root.join(...)` |
| P3-02 Versionierte KDF | ✔ bestätigt | `passwords_file_store.rs`: v2/600k, strikte Ablehnung unbekannter Parameter, v1-Lesepfad |
| P3-03 Fontmetrik-Layout | ○ laut Doku | Nicht selbst nachgemessen; Screenshot-Matrix inkl. 150 %-Zoom existiert |
| P3-04 Alle Dateien < 500 Zeilen | ✔ formal / ✘ Methode | Max. 495 Zeilen; aber 64 mechanische `_part_N`-Splits (M2) |
| P3-05 Zentrale Legacy-Pfade | ✔ bestätigt | `startup.rs:43` nutzt `correo_storage::current::legacy_roots()` |

---

## Kafka-Bewertung (Erweiterbarkeit nach 1.0)

**Grundlage ist gut, ein Schritt fehlt.**

Was trägt:
- **Adapter-Isolation:** `mqtt/adapter.rs` übersetzt App-Zustand → `MqttCommand`; rumqtt bleibt hinter `correo-mqtt` gekapselt. Ein `correo-kafka`-Crate kann dem Muster folgen.
- **Vertrag existiert:** `transport.rs` definiert `MessageEnvelope` (Adresse + Body + `DeliverySemantics` + namespaced Protokoll-Metadaten), `ConnectionCommand`, `TransportEvent`, `TransportPort` — genau die in P2-01 geforderte kleine Oberfläche, mit Tests.
- **Plugin-ABI v2 additiv:** Transport-Nachrichten laufen produktiv als `MessageEnvelope` durch die Plugin-Pipeline (`plugin_helpers.rs`, `plugins_abi_install.rs`); die v1-Bridge ist regressionsgetestet. Plugins müssen für Kafka nicht neu geschnitten werden.
- **Model-Muster:** `model/mqtt.rs` ist als protokollspezifisches Geschwistermodul geschnitten; ein `model/kafka.rs` fügt sich ein, ohne generische Schichten umzubauen.

Was fehlt bzw. bindet:
- **Der Port ist unbefahren (M4):** Kein MQTT-Produktionspfad läuft durch `ConnectionCommand`/`TransportEvent`/`TransportPort`. Erste Maßnahme vor jedem Kafka-Code: einen realen Pfad (Publish oder Incoming) durch den Port routen. Erst dann ist der Vertrag belastbar — Abnahmepunkt 9 aus `CODE_REVIEW.md` bleibt korrekt und offen.
- **`AppCommand`/`AppEvent` sind MQTT-geprägt:** `UpdatePublishQos(QosLevel)`, `SetPublishRetained`, Topic-Semantik (`commands.rs:86-109`). Für 1.0 richtig (kein generisches Über-Design); für Kafka werden Commands additiv erweitert oder pro Workspace-Typ dispatcht — kein Blocker, aber der Ort der künftigen Arbeit.
- **UI-Workbench:** Topic/QoS/Retained sind in Publish-/Subscribe-Panels verdrahtet, aber über Snapshot-DTOs entkoppelt — Kafka braucht eigene Panels (Partition/Key/Offset), keinen Umbau der bestehenden.

**Fazit Kafka:** Keine strukturellen Blocker. Die Architektur verlangt für Kafka additive Arbeit (neues Transport-Crate, neue Model-/UI-Module, Command-Erweiterung), keinen Umbau bestehender Schichten — unter der Bedingung, dass der Transport-Port vor dem Kafka-Start produktiv angebunden wird.

---

## Positive Evidenz (über CODE_REVIEW.md hinaus)

- **Kein Code bei den Splits verloren:** Symbol-Vollständigkeitsprüfung über alle 37 include-Stub-Dateien: 0 verlorene Symbole (4 nachweislich absichtliche Löschungen mit 0 Referenzen).
- **i18n vollständig:** Alle statisch referenzierten Schlüssel in allen 9 Locales vorhanden (skriptgeprüft).
- **TLS-Design vorbildlich:** „Hostname-Verifikation deaktivieren" toleriert ausschließlich `NotValidForName` und behält Chain-/Gültigkeitsprüfung (`transport/tls.rs`, `SkipHostnameVerification` delegiert an `WebPkiServerVerifier`) — Java-Parität ohne Blanko-Vertrauen; SSH nutzt TOFU als Produktions-Default.
- **Datei-Fallback-Store solide:** AES-256-GCM, frische Salt/Nonce pro Save, Nonce-Längenprüfung als behebbarer Fehler, leerer Bestand löscht die Datei, atomarer Write.
- **Packaging-Guard sauber:** Sidecar-Mengengleichheit, pro Artefakt neu berechnete SHA-256, `SHA256SUMS` zuletzt aus Verzeichnis-Scan, deterministische ZIPs mit expliziten Unix-Modes (0755/0644), testgedeckt.
- **Diagnostics-Ausbau maßvoll:** Rolling-File-Logging mit non-blocking Writer und degradierendem Fallback bei Setup-Fehler; Workspace verlor netto 7 Dependencies, `atomic-write-file` ist die einzige sicherheitsbegründete Neuaufnahme.
- **Disziplin der Deletion:** −21.798 gegen +9.432 Zeilen; tote Pfade (Connection-Launcher, doppelte WASM-Builds, Toolchain-Mutation) wurden entfernt statt umgebaut.

---

## Empfohlene Reihenfolge vor Release

1. **B1:** M2 entscheiden (echte Module oder Regel lockern), dann alle 99 Dateien committen; `cargo check --workspace --all-targets --locked` auf frischem Checkout.
2. **B2:** Script-Deadline konfigurierbar machen bzw. produkttauglich setzen (Produktentscheidung dokumentieren).
3. **H1–H3:** Asset-URLs dynamisch + Smoke-URL-Validierung; Publish-Gating auf alle Jobs; Installer-Pflicht im Package-Smoke.
4. **M1/M3/M5:** Plugin-Spec-Liste single-sourcen, Toolchain pinnen, Tag-vs.-Version-Assertion.
5. Die offenen Realabnahmen aus `CODE_REVIEW.md` (Windows-Fehlerinjektion, Dauerlast, Tag-Release-Smoke, reale Keyring-/TLS-Abnahme) bleiben unverändert gültig.
6. **Vor Kafka-Start:** einen MQTT-Produktionspfad durch `TransportPort` führen (Abnahmepunkt 9).
