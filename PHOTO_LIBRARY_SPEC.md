# Photo Library — Experience & Technical Spec

**Status:** Pre-implementation planning document. No code until this is approved.
**Phase:** 6 (mobile + sync sprint)
**Owner:** Rodrigo — review and approve before any implementation begins.

---

## The problem, stated plainly

People are paying Google and Apple monthly just to keep their own memories accessible. Their photos are scattered across dead phones, USB drives, old laptops, and current devices running out of space. The platforms that were supposed to solve this have turned it into a subscription. Older adults don't know how to fix it. Young people can't afford to pay indefinitely. Parents want family photos in one place but have incompatible devices.

This feature is the answer. Local-first photo library. Syncs across your devices on your network. No server. No subscription. No compression. No data leaving unless you say so.

---

## Core UX principles (read before designing any screen)

**1. No choices before value.** The first thing a user sees is their photos — not a settings screen, not a "where do you want to store them?" dialog. Configuration happens after the library exists.

**2. The app finds devices; the user doesn't.** mDNS autodiscovery means nearby devices appear automatically. The user taps "yes." They never type an IP address.

**3. Duplicates are invisible.** Perceptual hashing runs during import. Duplicates don't appear — they're silently skipped. One confirmation: "We found 847 duplicates. Skip them?" That's it.

**4. Sync just happens.** When a device is on the same WiFi as another, new photos sync in the background without any action. No "sync now" button. No reminder.

**5. Storage is concrete, not abstract.** Always show "4,312 photos · 18.4 GB" — never percentages. Users understand gigabytes better than abstract fractions.

**6. Language is human, not technical.** See the language table at the bottom of this document.

---

## Four user personas and their journeys

### Grandparent (Rosa, 68)
8 years of phone photos. Phone almost full. Terrified of losing them. Never opened a terminal.

Journey:
1. Opens app → sees "Your photos" with one big button: "Bring in my photos"
2. Camera roll offered automatically. Count shown: "4,312 photos found."
3. Confirms → import runs in background with progress. Done notification.
4. Laptop nearby → app shows "Found nearby: Rodrigo's laptop." One tap: "Sync photos here too."
5. Both devices have the photos. No subscription. No cable.

What she must never see: file paths, IP addresses, sync settings, conflict dialogs, storage backend choices.

### Young person (Mateo, 24)
Paying €3/month Google One. Photos on 3 dead phones + current phone + a USB drive. Wants to escape the subscription.

Journey:
1. Opens app → "You have photos in multiple places. Want to bring them together?"
2. Sources shown as checkboxes: current phone, USB drive, Google Takeout folder
3. Google Takeout: inline guide shown "here's how to download your Google photos first" — one link, three steps
4. Import runs. "Found 847 duplicates — skip them?" Yes.
5. Timeline shows unified library. "Freed 4.2 GB. You can cancel Google now."

Key design decision: the Google Takeout guide must be built into the onboarding, not linked externally. Many users don't know what Takeout is.

### Parent (Ana + partner, 38)
Photos split between iPhone and partner's Android. Old laptop in a closet. USB drive not opened in 2 years. Wants "family photos in one place."

Journey:
1. Creates "Family library"
2. Invites partner: app shows nearby devices. Partner taps "join." No accounts. No email.
3. Both phones import their own photos. Sync merges by date.
4. USB drive: drag folder onto app window (desktop) or cable connect (mobile) → import
5. New photos taken on either phone → appear on both within minutes on same WiFi

Key design decision: shared library discovery must not require entering names or codes. It's "devices on your network that have the app" — pick from a list.

### Developer (Carlos, 31)
Has a NAS. Wants proper architecture. No trusting companies.

Journey:
1. Settings → Library location → /mnt/nas/photos
2. All devices point to same NAS path
3. Quick Actions pipeline: "every Sunday, copy new photos to /mnt/backup/photos"
4. Done. Automated. Never thinks about it again.

What he needs: configurable library path, Quick Actions integration, export to any folder, no forced cloud.

---

## Import sources (all must work)

| Source | How | Notes |
|--------|-----|-------|
| Current device camera roll | Automatic offer on first launch | Android: MediaStore API. iOS: Photos framework (when Tauri iOS ships). |
| Folder on any device | Drag onto app window (desktop), browse dialog (mobile) | Recursive scan. All image formats. |
| USB drive / SD card | Appears as folder source when connected | Show device name, not path, for non-technical users. |
| Google Takeout archive | .zip or extracted folder | Built-in guide: how to download from Google. Preserve original dates from JSON metadata. |
| iCloud export | Extracted folder from Mac Photos export | |
| Another Eleutheria device on same network | mDNS discovery → select → sync | |
| Existing photo management apps | Not for Phase 6. Plugin territory. | Lightroom, Darktable etc. can be community plugins. |

---

## Duplicate detection

Every photo gets a perceptual hash (pHash) computed on import. This is a 64-bit fingerprint that's similar for visually identical images even if:
- The file was re-saved (different JPEG compression)
- Metadata was stripped
- The file was renamed
- Resolution was slightly changed

Threshold: pHash Hamming distance ≤ 10 = duplicate. Present to user as one photo, not two.

Edge cases:
- Near-identical burst photos (person blinking): NOT treated as duplicates. Hamming distance > 10.
- Screenshot of a photo: NOT treated as duplicate. Different content.
- Same photo at two resolutions: IS treated as duplicate. Keep the higher resolution.

The dedup index lives in SQLite with the library. It persists across app restarts so re-importing from the same source doesn't re-process.

---

## Storage format

Photos are stored in their original format, never recompressed. The folder structure is:

```
[library root]/
  2024/
    03/
      15/
        IMG_4721.jpg         ← original, untouched
        IMG_4721.jpg.thumb   ← 400px thumbnail, generated locally
  2023/
    ...
  .eleutheria/
    library.db               ← SQLite index (photos, hashes, albums, faces future)
    thumbs/                  ← thumbnail cache
```

Why this structure:
- Year/Month/Day is universally understood by non-technical users
- Each folder is browsable in any file manager without the app
- Thumbnails are separate so they can be regenerated without touching originals
- The SQLite index can be rebuilt from the folder structure if it gets corrupted

Never do:
- Rename original files
- Convert formats (no HEIC → JPG without explicit user request)
- Change EXIF data
- Store photos in an opaque database blob

---

## Local-network sync

### Discovery
Uses mDNS (Multicast DNS / Zeroconf / Bonjour — same protocol, different names on different platforms). When the app starts, it announces itself on the local network. Other devices running the app see it appear automatically.

Technical: `_eleutheria._tcp.local.` service type. Implemented via the `mdns-sd` Rust crate (MIT license, actively maintained).

### Sync protocol
Pull-based. Each device maintains a monotonically increasing `sequence_number` for its library changes. When two devices meet on the same network:

1. Device A asks Device B: "what's your sequence number?"
2. If B's sequence > A's last-seen-B-sequence, A asks for new items since that sequence
3. B sends photo metadata (hash, date, path on B, size) — not the photo itself
4. A checks its own hash index. Any hashes A doesn't have → A requests the file
5. File transferred over local HTTP (same Axum server, new `/api/photos/` routes)
6. A stores the file, updates its index, updates its last-seen-B-sequence

This means:
- Sync is incremental — only new/changed photos transfer
- No central server needed
- Works across Android ↔ iOS ↔ Windows ↔ macOS ↔ Linux
- If a device is offline during sync, it catches up the next time it's on the same network

### Conflict resolution (Phase 6)
Photos are immutable once imported — they don't change. True conflicts don't exist. What can happen:
- Same photo on two devices (same hash) → deduplicated, one copy kept
- Different photos with same filename → both kept, one renamed with suffix

Deleted photos: deletion is soft-delete in Phase 6 (30-day trash, same as notes). Hard delete propagates to other devices only after user confirms "delete everywhere."

### What syncs
Phase 6: photos, albums, favorites.
Phase 7: edits, annotations, freehand notes on photos.
Never syncs automatically: settings, theme, other app data — those stay per-device unless user explicitly exports.

---

## The "safe on N devices" indicator

Every user, regardless of technical level, needs to feel confident their photos won't disappear. The library panel always shows:

```
4,312 photos · 18.4 GB
Safe on 2 devices  ●●○  (2 of 3 devices synced)
Last synced: 4 minutes ago
```

When only 1 device has the photos:
```
Safe on 1 device  ●○○
Tip: sync to another device so your photos are protected if this one breaks.
```

This replaces "backup" (technical) with "safe on N devices" (human). The concept is the same but the language is about outcomes, not processes.

---

## Onboarding flow (non-technical user path)

```
First launch of Photos tab
        │
        ▼
"Your photos aren't here yet."
[Large button: "Bring in my photos"]
        │
        ▼
Source selection (shown as recognizable icons, not paths):
  ● This phone's camera roll    ← auto-detected, count shown
  ○ A folder or USB drive
  ○ Google Photos (guide included)
  ○ Another device on my network
        │
        ▼
Confirm: "Import 4,312 photos from your camera roll?"
[Yes, import them]   [Not now]
        │
        ▼
Import progress (background, dismissable)
"Importing your photos... 1,204 of 4,312"
[You can use the app while this runs]
        │
        ▼
Done notification: "Your 4,312 photos are here."
        │
        ▼
If another device found on network:
"Rodrigo's laptop is nearby. Copy photos there too for extra safety?"
[Yes, sync now]   [Not now]
```

At no point in this flow does the user see: file paths, IP addresses, sync protocols, storage backends, compression settings.

---

## Advanced settings (power user, hidden by default)

Accessible via Settings → Photos (advanced):

- Library location (default: app data dir)
- Sync: on/off per device
- Sync: WiFi only vs mobile data
- Dedup threshold (default: 10, range 0–20)
- Thumbnail size (default: 400px)
- Auto-import: on new photos from camera roll (default: on)
- Export to external backup path (Quick Actions compatible)
- Conflict resolution policy: keep both / keep larger / keep newer

---

## What this is NOT

- Not a photo editor (basic rotate/crop is fine; that's it for Phase 6)
- Not a face recognition system (privacy implications, complexity; future plugin territory)
- Not a cloud service (we don't run servers)
- Not a replacement for professional photo management (Lightroom, Darktable — community plugins)
- Not a video library (videos can be in the library folder but no special handling in Phase 6)

---

## Language table — what to say and what not to say

| Never say | Always say |
|-----------|-----------|
| Configure sync endpoints | Set up on another device |
| Resolve merge conflict | Found the same photo in two places |
| Perceptual hash deduplication | We skipped X duplicate photos |
| mDNS peer discovery | Found nearby: [device name] |
| Select storage backend | Choose where to keep your photos |
| Library database | Your photo index |
| Sequence number | Last synced |
| Incremental sync | Only new photos will transfer |
| Backup | Keep a copy |
| Safe on N replicas | Safe on N devices |

---

## Technical implementation notes (for Claude Code / Cursor)

### New Rust crates needed (verify before adding)
- `mdns-sd` — mDNS service discovery. MIT. Check Rust 1.92 + Tauri 2.x compatibility.
- `img-hash` — perceptual hashing. MIT. Pure Rust, no system deps.
- `kamadak-exif` — EXIF reading. MIT. For date extraction and preservation.
- `image` crate — already likely present; needed for thumbnail generation.

### New SQLite tables (new migration file)
```sql
CREATE TABLE photos (
  id          TEXT PRIMARY KEY,        -- UUID
  phash       INTEGER NOT NULL,        -- 64-bit perceptual hash
  file_path   TEXT NOT NULL,           -- relative to library root
  original_filename TEXT NOT NULL,
  captured_at INTEGER,                 -- Unix timestamp from EXIF or file mtime
  imported_at INTEGER NOT NULL,
  file_size   INTEGER NOT NULL,
  width       INTEGER,
  height      INTEGER,
  device_id   TEXT NOT NULL,           -- which device originally imported this
  deleted_at  INTEGER DEFAULT NULL     -- soft delete
);
CREATE INDEX photos_phash ON photos(phash);
CREATE INDEX photos_captured_at ON photos(captured_at);
CREATE INDEX photos_device ON photos(device_id);

CREATE TABLE photo_albums (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  created_at  INTEGER NOT NULL
);
CREATE TABLE photo_album_members (
  album_id    TEXT NOT NULL REFERENCES photo_albums(id) ON DELETE CASCADE,
  photo_id    TEXT NOT NULL REFERENCES photos(id) ON DELETE CASCADE,
  PRIMARY KEY (album_id, photo_id)
);

CREATE TABLE sync_state (
  device_id   TEXT NOT NULL,
  last_seq    INTEGER NOT NULL DEFAULT 0,  -- last sequence we received from this device
  PRIMARY KEY (device_id)
);
```

### New Axum routes
```
GET  /api/photos                    → paginated photo list (timeline)
GET  /api/photos/:id/thumb          → thumbnail (generate on demand, cache)
GET  /api/photos/:id/full           → full resolution file
POST /api/photos/import             → start import job (folder path)
GET  /api/photos/import/progress    → import job progress (SSE or polling)
GET  /api/photos/sync/peers         → list of discovered mDNS peers
POST /api/photos/sync/:device_id    → trigger sync with specific device
GET  /api/photos/sync/sequence      → return our current sequence number (used by peers)
GET  /api/photos/sync/since/:seq    → return photo metadata since sequence N (for peers)
GET  /api/photos/sync/file/:id      → serve a photo file to a peer (authenticated)
```

### MCP tools to expose
```
list_photos(limit, offset, date_from, date_to) → photo metadata list
search_photos(query) → photos matching date/album/filename
get_photo(id) → photo metadata + file path
import_photos(folder_path) → start import job
get_library_stats() → count, size, device count, last sync
```

### Platform-specific camera roll access
- Android: `tauri-plugin-fs` with `MANAGE_EXTERNAL_STORAGE` permission, or MediaStore content resolver via Tauri commands. Research required — document in a DECISIONS.md entry.
- iOS: Photos framework via Tauri plugin. Blocked on Tauri iOS stable.
- Desktop: Folder picker via `tauri-plugin-dialog`.

### Thumbnail generation
Generate on first request, cache in `.eleutheria/thumbs/` subdirectory. 400px longest edge, JPEG 85% quality. Background tokio task — never blocks the UI.

### Import job architecture
Long-running import (thousands of photos) must:
1. Run in `tokio::spawn` — never block Axum handlers
2. Report progress via polling endpoint (`GET /api/photos/import/progress`)
3. Be cancellable
4. Resume if app is closed and reopened (track import state in SQLite)
5. Handle errors per-file (one bad JPEG doesn't stop the whole import)
