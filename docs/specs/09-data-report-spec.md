# Data, Session and Report Specification

## Session identity
Each session has UUID, application version, ruleset version, start/end timestamps, edition, host OS/build, machine label (optional), selected device/backend and environment profile.

## Event record
Fields:
- monotonic timestamp
- wall timestamp
- category
- severity
- source
- device/endpoint/channel IDs
- numeric values with units
- message
- correlation ID

## Measurement record
Fields:
- test ID/version
- method
- stimulus definition if any
- sample rate/buffer/sample format
- routing
- start/end
- values
- uncertainty/quality flags
- validity
- thresholds
- result state
- evidence references

## Report outputs
Human report, JSON and CSV tables. JSON is canonical machine-readable export. A later schema version may add fields but must not silently change units or meanings.

## Privacy
No audio is stored by default. Process names in DAW mode are local report metadata. Raw audio retention requires explicit enablement and a visible recording indicator.

## Retention
User-configurable session retention. Deleting a session removes associated raw captures from the local application store.

## Comparison
Only compare like-for-like measurements when test method, routing class and key configuration are compatible. Otherwise present side-by-side without percentage claims.

## Support bundle
Optional ZIP bundle may contain report JSON, human report, application log, anonymized device capability inventory and selected raw evidence. It excludes unrelated user files.
