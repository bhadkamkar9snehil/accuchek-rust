# Accu-Chek backend research and invariants

This backend is intentionally local-only and read-only toward the meter. The goal is faithful acquisition of source measurements before any analytics are calculated.

## Reference implementations reviewed

| Project | Role in verification | Relationship |
| --- | --- | --- |
| `tidepool-org/uploader` (`lib/drivers/roche/accuChekUSB.js`) | Maintained Roche IEEE 11073/WebUSB state machine, MDS metadata parsing, transfer retry/error behavior, high/low sentinels | Primary maintained Roche reference |
| `tidepool-org/windows-driver` / uploader `resources/win/phdc.inf` | Windows WinUSB PHDC binding and supported Roche VID/PIDs | Driver reference |
| `emogenet/accuchek` | Native C++/libusb implementation of the same meter protocol | Native transport implementation, originally informed by Tidepool; issue history independently confirms PID `0x21d7` works |
| `davdiv/accu-chek` | TypeScript/WebUSB implementation, model mapping, parsing behavior | Derivative of Tidepool with modifications; useful cross-check, not independent protocol evidence |
| `cikeZ00/accuchek-rust` | Original Rust/libusb application forked here | Starting implementation, largely ported from the C++ implementation |
| `signove/antidote` | Generic IEEE 11073 manager/PM-Store implementation | Standards-level cross-check only; not Roche-specific |
| Nightscout `xDrip`, `Juggluco`, `nov-open-reader` | Other IEEE 11073 PM-Store clients (not glucose-meter USB) | Protocol-adjacent evidence only; useful for APDU/PM-Store concepts, not Roche device behavior |

GitHub forks/copies of `emogenet/accuchek` (for example `force-focus/accuchek_usb` and multiple `*/accuchek` repositories) are not counted as independent implementations. Pump/CGM projects using the Accu-Chek name are also excluded because they do not exercise the Instant USB PHDC path.

## Confirmed USB family

Roche USB vendor ID is `0x173A` for this PHDC family. Tidepool's current driver/INF maps:

- `0x21CF` — Accu-Chek Aviva Connect
- `0x21D5` — Accu-Chek Guide
- `0x21D6` — Accu-Chek Guide Me
- `0x21D7` — Accu-Chek Instant
- `0x21D8` — ReliOn Platinum
- `0x21DB` — Accu-Chek Guide Link

`emogenet/accuchek` issue #1 independently reports that product ID `0x21d7` works with the libusb implementation. The same issue also contains examples from older Roche hardware demonstrating that the measurement status word can be non-zero on valid measurements.

The application matches an explicit VID/PID whitelist first. USB descriptor shape alone must never be enough to identify a medical device.

## Windows driver policy

The preferred Windows path is the Roche/Tidepool PHDC WinUSB binding. Tidepool's installer and INF bind the supported meters to Microsoft's WinUSB stack, including Instant `VID_173A&PID_21D7`. Do not require Zadig when a compatible WinUSB binding is already installed. Zadig is a troubleshooting fallback because replacing a vendor driver can affect other Roche software.

## Protocol invariants

The manager performs only the operations needed to read stored measurements:

1. Standard USB GET_STATUS.
2. Receive IEEE 11073 association request.
3. Send association response.
4. Receive/acknowledge extended configuration and find PM-Store.
5. Read MDS attributes for model, production serial and meter wall-clock time.
6. Query segment information.
7. Trigger segment transfer.
8. Parse every page, acknowledge every page, stop only on the protocol end flag.
9. Best-effort association release and interface release on both success and failure.

The backend does **not** set the meter clock, clear records, change settings, or otherwise mutate clinical source data.

## Source-fidelity rules

- Preserve the raw 16-bit glucose value.
- Preserve the raw IEEE 11073 measurement-status word even when its bit semantics are not yet decoded.
- Never silently drop a record because `status != 0`.
- Non-zero status is **not** treated as invalid. Published Accu-Chek records contain ordinary measurements with non-zero status words, and Tidepool currently imports measurements regardless of that word. All records remain analytically visible unless a specific exclusion bit is positively identified and tested.
- Preserve meter-local wall-clock timestamps without pretending the meter supplied a UTC offset.
- Preserve device identity (Roche model/serial when available) and deduplicate per device.
- Preserve repeated identical readings in the same meter minute using an occurrence index; repeat syncs remain idempotent.
- Roche sentinel `0x07FE` is retained as raw value and normalized to a display/analysis value above 600 mg/dL; `0x0802` is retained and normalized below the known range. Raw value and range state remain available for audit.

## Defensive parsing

All fixed-offset reads must be length-checked. Malformed/truncated USB frames must return typed errors rather than panic. The transfer has a hard page limit to prevent an endless loop if an end flag never arrives. Short writes, empty reads, unexpected APDU/action types and meter error codes are treated as failures.

Tidepool retries the configuration stage once because a meter can occasionally return an unusable configuration frame immediately after association; this backend mirrors that behavior.

## Known open question: measurement status/meal flags

Tidepool issue #1708 (opened 2025-11-22) documents that before-meal, after-meal and sleep flags are not currently surfaced by its Accu-Chek USB driver. The `emogenet/accuchek` issue history also shows valid measurements with status values such as `0x55`, `0x41`, `0x15`, `0x29`, `0x47`, `0x49` and `0x26`, reinforcing that non-zero is contextual rather than a blanket invalid marker.

Until those bits/fields are verified against captures or specification material, this project stores the raw status word and does not invent interpretations.

## Storage integrity

Schema v2 removes the legacy global `epoch UNIQUE` identity. Measurements are keyed by device + meter timestamp + raw value + status + occurrence. Existing databases are migrated in-place, retaining IDs, notes, tags and import timestamps. SQLite uses WAL and `synchronous=FULL`.

Database-at-rest encryption is a separate release gate: before a privacy-grade public release, the desktop layer should use SQLCipher with a key protected by the operating-system credential mechanism (DPAPI on Windows), with an explicit migration path from plaintext databases.

## Analytics terminology

These are intermittent finger-stick measurements, not a continuous glucose monitor stream. Analytics should describe **percentage of sampled readings in range**, not continuous “time in range,” unless a future data source actually provides continuous measurements.
