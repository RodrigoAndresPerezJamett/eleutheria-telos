# Local Network Sync — Experience & Technical Spec

**Status:** Pre-implementation planning. No code until approved.
**Phase:** 6
**Covers:** Clipboard, Notes, Captures, Photos (photos has its own spec — reference that for storage format)

---

## The problem

People live across multiple devices. A URL copied on the phone isn't on the laptop. A note written on the tablet isn't on the desktop. A photo taken yesterday isn't on any machine except the one in their pocket. The platforms that solved this — iCloud, Google Drive, Samsung Cloud — all require accounts, send data to servers, and are locked to their own ecosystems.

This feature solves it locally. Same WiFi = everything is everywhere. No account. No server. No data leaving the house.

---

## Core UX principles

**1. It just starts working.** When two devices running the app are on the same WiFi, sync begins automatically. No pairing, no code, no confirmation — unless the user is adding a device to a shared library for the first time.

**2. Trust is device-level, not account-level.** The first time Device B appears on Device A's network, Device A shows: "Rodrigo's phone wants to sync. Allow?" Once allowed, that device is trusted forever. No re-authentication on every session.

**3. Conflicts are invisible for most cases.** Notes and clipboard entries are immutable after creation (you append, you don't edit the past). Photos don't change after being taken. The only real conflict is: same note edited on two devices while offline. That case is handled visibly, not silently.

**4. Status is always honest.** The user always knows how many devices their data is on. "Safe on 2 devices" is shown everywhere. When sync is behind, the UI says so clearly and non-alarmingly.

**5. Offline is normal, not an error.** The app works perfectly with no other devices present. Sync is opportunistic — when devices meet, they catch up. Being offline for a week and then reconnecting is a supported use case, not an edge case.

---

## What syncs and what doesn't

| Data | Syncs | Notes |
|------|-------|-------|
| Clipboard entries | Yes | New entries only — deletions don't propagate in Phase 6 |
| Notes (content) | Yes | Full two-way sync with conflict detection |
| Note tags and backlinks | Yes | Derived from content, so syncs with content |
| Captures (OCR results, transcriptions) | Yes | Append-only, no conflicts possible |
| Photos | Yes | See PHOTO_LIBRARY_SPEC.md |
| App settings (theme, glass, etc.) | No — Phase 7 | Per-device in Phase 6 |
| Pipeline definitions (Quick Actions) | No — Phase 7 | Pipelines are device-specific in Phase 6 |
| AI model files | No | Too large; user downloads per-device |
| Plugin data | No | Plugin-sandboxed; sync is plugin's responsibility |

---

## User journeys

### Journey 1: First time, two devices, same person (most common)
Ana has the app on her laptop and installs it on her phone.

1. She opens the app on her phone for the first time.
2. Within 10 seconds, a notification appears on both devices: "Rodrigo's laptop found on your network. Sync with it?"
3. She taps Yes on both.
4. Initial sync runs in background — progress shown as "Bringing over 847 notes and 3,241 clipboard entries."
5. Done. From now on: new note on phone → on laptop within seconds. New clipboard copy on laptop → on phone within seconds. Nothing else to configure.

### Journey 2: Shared library — family or couple
Ana and her partner both have the app. They want shared Notes for shopping lists and shared photos.

1. Ana opens Settings → Sync → "Add a shared device."
2. Her partner's device appears (by name). She taps it.
3. Her partner gets a notification: "Ana's phone wants to share Notes and Photos with you. Allow?"
4. He taps Allow and selects which data to share (not everything — just Notes tagged #family and Photos).
5. Now: shared notes sync between their devices. Private notes stay private.

Key detail: granular sharing is important. Not everything should sync to a partner's device automatically — only what the user explicitly chooses.

### Journey 3: Returning after being offline
Carlos has been traveling for 2 weeks without WiFi at home. He's been taking notes and photos on his phone.

1. He gets home, both devices connect to the same WiFi.
2. Sync starts automatically in background.
3. A small indicator appears: "Syncing 2 weeks of updates — 1,204 items."
4. Completes within a few minutes. No action required.

### Journey 4: Conflict — same note edited on two devices
Ana edited her "Grocery list" note on her phone (added eggs) while her laptop was offline. She also edited it on her laptop (added milk). Now the laptop reconnects.

1. The app detects the conflict.
2. A small badge appears on the note: "Edited on 2 devices."
3. She taps it — sees a simple merge view: left side is phone version, right side is laptop version.
4. She taps "Keep both changes" — the app merges the two versions (union of lines).
5. Or she taps "Use phone version" / "Use laptop version."

The merge view must be simple enough that a non-technical user understands it. No git diff UI. Just "here's what each device had — which do you want?"

### Journey 5: Removing a device
Ana sells her old tablet. She wants to remove it from sync.

1. Settings → Sync → Trusted Devices → "iPad (last seen 3 months ago)"
2. "Remove this device." Confirmed.
3. The device is removed from the trusted list. If the tablet ever reconnects, it will be treated as a new unknown device and ask for permission again.

---

## Trust model

### Device identity
Each installation generates a persistent UUID and a keypair on first launch. The public key is the device's identity. Device name comes from the OS hostname, editable in Settings.

### Trust establishment
First connection between two devices requires mutual confirmation — both sides tap "Allow." After that, the public key is stored in the trusted devices list. Future connections are automatic (no UI) as long as the public key matches.

### Trust is scoped
When trusting a device, the user chooses what to share:
- Everything (same-person devices)
- Selected data types (Notes, Photos, Clipboard — individually toggled)
- Read-only (the other device can receive but not send changes)

This is how the "partner shared library" use case works without accidentally syncing private notes.

### Revocation
Removing a device from the trusted list immediately stops sync. The removed device's data is not deleted — it stays on both devices. Only future changes stop propagating.

---

## Protocol design

### Discovery
mDNS (Multicast DNS / Zeroconf / Bonjour). Service type: `_eleutheria._tcp.local.`

Each device announces:
- Device UUID
- Device name (human-readable)
- App version
- Protocol version

Discovery is passive — the app listens for announcements. No polling. When a device appears or disappears, the app knows within ~2 seconds.

### Sync protocol: vector clocks + append-first

Each device maintains a **vector clock** — a map of `{device_uuid: sequence_number}` per data type. This is the fundamental mechanism that enables sync without a central server.

When devices meet:
1. Device A sends its vector clock to Device B.
2. Device B compares and identifies what A is missing (items where B's sequence > A's last-seen-B-sequence).
3. B sends those items to A.
4. A sends its items to B similarly.
5. Both update their vector clocks.

This is purely additive in Phase 6. Deletions are soft (items get a `deleted_at` flag) and the deletion propagates like any other change.

### Conflict detection
A conflict exists when the same item (same UUID) has been modified on two devices since their last sync. Detection:

```
Last sync between A and B: vector clock snapshot stored
Item X last modified on A at sequence 45
Item X last modified on B at sequence 12 (since last sync)
→ Conflict: both devices modified X independently
```

Non-conflicting edits (only one device touched the item since last sync) merge automatically.

### Transport
All sync traffic uses the existing Axum HTTP server on localhost, extended with new routes at `/api/sync/`. Encryption: TLS with the device keypair certificate. No plaintext sync, even on the local network.

New routes:
```
GET  /api/sync/identity          → {uuid, name, pubkey, protocol_version}
GET  /api/sync/clock             → vector clock for all data types
POST /api/sync/pull              → body: {data_type, since_clock} → items since that clock
POST /api/sync/push              → body: {data_type, items} → receive items from peer
GET  /api/sync/peers             → list of currently visible mDNS peers
POST /api/sync/trust             → body: {peer_uuid, data_types} → establish trust
DELETE /api/sync/trust/:uuid     → revoke trust
```

### Bandwidth and batching
- Sync runs in `tokio::spawn` — never blocks the UI or any other operation.
- Items are batched: max 100 items per HTTP request, chunked for large payloads.
- Photos are NOT included in the standard sync pull/push — they use the photo-specific transfer routes from PHOTO_LIBRARY_SPEC.md.
- Clipboard entries are compressed (zstd) before transfer if > 1KB.
- Notes are sent as full content (not diffs) in Phase 6. Diff-based sync is Phase 7+.

---

## Edge cases and how to handle them

### Device joins network mid-session
Auto-detected via mDNS. Sync starts within 5 seconds of discovery. No user action needed if device is already trusted.

### Very large initial sync (new device with years of data)
Progress shown clearly: "Syncing — 3,241 of 8,420 items." Runs fully in background. App is usable during sync. If interrupted (WiFi drops), sync resumes exactly where it left off on next connection — no re-transfer of already-synced items.

### Two devices edit a note simultaneously (both online, race condition)
Last-write-wins for the content. A `modified_at` timestamp (microsecond resolution) determines which version wins. The losing version is saved as a conflict copy (never discarded silently).

### Network is slow (congested WiFi, lots of devices)
Sync uses adaptive backoff. If a sync round takes > 30 seconds, the next round waits 60 seconds before trying again. Exponential backoff up to 10 minutes. Status shown: "Sync is taking longer than usual."

### One device has corrupt data
Corrupt items (failed SQLite reads, malformed JSON) are skipped during sync with a warning logged. Corrupt data on one device does not propagate to others. The user sees: "3 items couldn't sync — tap to see details."

### Clock skew between devices
Physical clocks can differ between devices. All sync logic uses **logical clocks** (sequence numbers, not wall time) for ordering. Wall time is stored for display only (when you see a note was written), not for sync ordering.

### User deletes a note that has already synced to another device
Soft delete (existing trash system). The deletion propagates as a `deleted_at` update to all trusted devices. The note moves to trash on all devices simultaneously. Permanent deletion (purge) only propagates after the user explicitly purges on the originating device — the other devices will purge on their next sync.

### App is uninstalled from a device
The device stops announcing on mDNS. Other devices notice it's gone within ~2 minutes. No cleanup needed — the device just stops syncing. Its data remains on other devices.

### Two different users on the same network (coffee shop, office)
Devices only sync with explicitly trusted devices. An unknown device on the same network cannot see your data — not your device list, not your items. The mDNS announcement only reveals: device name, UUID, protocol version. No data is accessible without trust establishment.

---

## SQLite changes needed

```sql
-- Device identity (one row per installation)
CREATE TABLE sync_identity (
  device_uuid    TEXT PRIMARY KEY,
  device_name    TEXT NOT NULL,
  public_key     TEXT NOT NULL,   -- PEM-encoded
  private_key    TEXT NOT NULL,   -- PEM-encoded, stored locally only
  created_at     INTEGER NOT NULL
);

-- Trusted peers
CREATE TABLE sync_peers (
  peer_uuid      TEXT PRIMARY KEY,
  peer_name      TEXT NOT NULL,
  public_key     TEXT NOT NULL,
  trusted_at     INTEGER NOT NULL,
  last_seen_at   INTEGER,
  sync_clipboard INTEGER NOT NULL DEFAULT 1,   -- booleans
  sync_notes     INTEGER NOT NULL DEFAULT 1,
  sync_captures  INTEGER NOT NULL DEFAULT 1,
  sync_photos    INTEGER NOT NULL DEFAULT 1,
  read_only      INTEGER NOT NULL DEFAULT 0
);

-- Vector clock: what sequence numbers we've received from each peer
CREATE TABLE sync_clock (
  peer_uuid      TEXT NOT NULL,
  data_type      TEXT NOT NULL,   -- 'clipboard' | 'notes' | 'captures' | 'photos'
  last_seq       INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (peer_uuid, data_type)
);

-- Sequence numbers for our own items (so peers know what to request)
-- Add to existing tables:
ALTER TABLE clipboard ADD COLUMN sync_seq INTEGER DEFAULT NULL;
ALTER TABLE notes     ADD COLUMN sync_seq INTEGER DEFAULT NULL;
-- (and a trigger to auto-assign seq on insert/update)

-- Conflicts awaiting user resolution
CREATE TABLE sync_conflicts (
  id             TEXT PRIMARY KEY,
  data_type      TEXT NOT NULL,
  item_id        TEXT NOT NULL,
  local_content  TEXT NOT NULL,   -- JSON snapshot of local version
  remote_content TEXT NOT NULL,   -- JSON snapshot of remote version
  remote_device  TEXT NOT NULL,
  detected_at    INTEGER NOT NULL,
  resolved_at    INTEGER
);
```

---

## New Rust crate needed

- `mdns-sd` (MIT, actively maintained) — mDNS service discovery. Verify Rust 1.92 + Tauri 2.x compatibility before adding. Add DECISIONS.md entry.
- `rustls` + `rcgen` — TLS for sync transport. `rcgen` generates self-signed certificates from the device keypair. Both MIT.
- `zstd` — compression for large clipboard entries. MIT.

---

## What this is NOT

- Not cloud sync (no server, not routed over the internet)
- Not end-to-end encrypted cloud backup (that's Phase 7+ optional feature)
- Not real-time collaborative editing (conflict resolution is async, not OT/CRDT)
- Not available across different networks without optional cloud sync (Phase 7+)
