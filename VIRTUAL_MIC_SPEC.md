# Virtual Microphone & Voice Effects — Experience & Technical Spec

**Status:** Pre-implementation planning. No code until approved.
**Phase:** 5 (gated on R1–R3 audio research)

---

## Honest answer to "can we make this OS-independent?"

**The DSP pipeline is fully cross-platform. The virtual device registration is not — and cannot be.**

Here is why: a virtual microphone must appear in the OS's audio device list so that Discord, Zoom, OBS, and other apps can select it as their microphone input. That device list is managed by the OS audio subsystem. Registering a new device in that list requires OS-level integration on every platform. There is no pure-Rust, no-OS-involvement way to do this — it would be equivalent to writing an audio driver in userspace, which is exactly what each platform requires.

However, the picture is much better than "install VB-Cable on Windows":

| Platform | Mechanism | Requires user action? | Kernel driver? |
|----------|-----------|----------------------|----------------|
| **Linux** | PipeWire `pw-loopback` subprocess | None — automatic on first use | No — pure userspace |
| **macOS** | CoreAudio `AudioServerPlugin` | None — bundled in .dmg, loads without reboot | No — userspace plugin since Big Sur |
| **Windows** | WASAPI loopback device | Yes — user runs a one-time installer | Yes — kernel driver (VB-Cable or equivalent) |
| **Android** | AudioEffect API + MediaProjection | App permission grant only | No |

**The architecture decision:** Build 100% of the audio processing (DSP pipeline, effects, mixing) in pure Rust using CPAL. This is genuinely cross-platform. The only OS-specific code is the final step: registering the virtual output device. That step is isolated in one module per platform, cleanly separated from the processing pipeline.

The Windows situation is the honest limitation. Document it clearly in the UI: "Windows requires a one-click driver install (free). This only needs to happen once." This is not a dealbreaker — many professional audio tools on Windows require the same thing (VB-Cable is a widely trusted, free tool with millions of installations). The key is that the app guides the user through it rather than leaving them to figure it out alone.

---

## What this feature is

A real-time audio processing pipeline that:
1. Captures audio from the user's real microphone (or any audio source)
2. Applies user-selected effects (pitch shift, noise reduction, voice effects, etc.)
3. Routes the processed audio to a virtual microphone device
4. Other apps (Discord, Zoom, OBS, games) see the virtual mic and use the processed audio

Use cases:
- Stream with voice effects (robot, radio, deeper voice)
- Remove background noise during calls
- Route soundboard audio through the mic
- Apply EQ to a bad microphone
- Play audio from the app (soundboard) through the mic in a call

---

## User journeys

### Journey 1: Non-technical user — "I want to sound better on calls" (most common)
Sofia uses Zoom for work. Her USB microphone sounds thin. She wants noise reduction and a slightly warmer tone.

1. Opens Voice Effects panel.
2. Sees her microphone is already selected (detected automatically).
3. Turns on "Noise reduction" — toggle. Sees a before/after waveform preview.
4. Adjusts "Warmth" — a single slider. Not "low shelf EQ at 200Hz."
5. Taps "Start." A system notification: "Eleutheria Mic is now available."
6. Opens Zoom, selects "Eleutheria Microphone" from Zoom's audio settings.
7. Done. Her voice on Zoom is now processed.

What she must never see: "FFT size," "sample rate," "buffer frames," "loopback routing," "null sink."

### Journey 2: Streamer — "I want voice effects and soundboard through mic"
Marcus streams games. He wants a robot voice effect and to play sound effects through his mic during streams.

1. Opens Voice Effects panel.
2. Selects "Robot" from the effects presets.
3. Adjusts intensity with one slider.
4. Goes to Soundboard panel — sees the same virtual mic is available as output target.
5. Sets OBS to use "Eleutheria Microphone."
6. Now: his voice is processed, and soundboard buttons play through the same virtual mic.

### Journey 3: Windows user — first-time setup
James installs the app on Windows. He opens Voice Effects.

1. Panel shows: "Voice Effects needs a one-time setup on Windows."
2. Clear explanation: "Windows requires a small audio driver to route processed audio to other apps. This is free and takes about 30 seconds."
3. [Install audio driver] button. Clicking it downloads and silently installs VB-Cable (or an equivalent bundled with the app installer in Phase 6+).
4. After install: "Setup complete. Voice Effects is ready."
5. Normal experience from here.

The driver install should happen inside the app — the user should not need to visit a third-party website.

### Journey 4: Privacy-conscious user who wants noise reduction only
Elena doesn't want a virtual mic visible to other apps. She just wants the app to filter her microphone before the audio reaches her voice recorder.

This is handled by routing processed audio directly to a file or to the app's own Voice recorder — no virtual device needed. A toggle: "Route to: [Virtual microphone] / [Voice recorder only]."

---

## Effects library

Effects are presented to users with friendly names, not technical names. Under the hood they are DSP operations.

| User label | Technical implementation | Difficulty |
|-----------|--------------------------|------------|
| **Noise reduction** | Spectral subtraction + RNNoise (open source neural model) | Medium |
| **Warmth** | Low-shelf boost + slight compression | Easy |
| **Clarity** | High-shelf boost + gentle gate | Easy |
| **Pitch shift** | Phase vocoder or rubberband library | Hard |
| **Robot** | Ring modulation + pitch correction | Medium |
| **Radio** | Bandpass filter (300–3400Hz) + light distortion + noise floor | Easy |
| **Deep voice** | Pitch shift down + slight reverb | Hard (depends on pitch shift) |
| **Reverb** | Convolution with impulse responses (offline, bundled) | Medium |
| **Noise gate** | Amplitude threshold gate | Easy |
| **Compressor** | Soft-knee dynamic range compression | Medium |

**Phase 5 ships:** Noise reduction, Noise gate, Warmth, Clarity, Radio, Compressor. These cover 90% of use cases and avoid the hardest implementation (pitch shift).

**Phase 6+ (after R1 CDP research):** Pitch shift, Robot, Deep voice, Reverb. These benefit from CDP or a dedicated DSP library.

---

## DSP architecture (pure Rust, cross-platform)

### Audio pipeline

```
[Real Microphone]
       │ CPAL input stream
       ▼
[Input Buffer]
  sample rate normalization
  channel conversion (mono/stereo)
       │
       ▼
[Pre-processing chain]
  1. Noise gate (if enabled)
  2. High-pass filter (remove subsonic rumble)
       │
       ▼
[Effects chain] (user-configurable order)
  ├── Noise reduction (RNNoise)
  ├── EQ (warmth / clarity)
  ├── Compressor
  ├── Pitch shift (Phase 6+)
  ├── Voice effects (robot, radio, etc.)
  └── Reverb (Phase 6+)
       │
       ▼
[Output Buffer]
       │
  ┌────┴─────────────────┐
  ▼                       ▼
[Virtual Device]    [File / Voice recorder]
  (OS-specific)      (direct, no OS needed)
```

Everything above the "Virtual Device" box is pure Rust, cross-platform, no OS dependency.

### Key Rust crates

- `cpal` (MIT) — cross-platform audio I/O. Input from real mic, output to virtual device (or file).
- `rubato` (MIT) — sample rate conversion. Handles the case where real mic and virtual device have different sample rates.
- `rnnoise-rs` or `nnnoiseless` — Rust bindings to RNNoise for noise reduction. Check license (RNNoise is BSD). Verify Python 3.14 / Rust 1.92 compatibility.

All DSP (EQ, gate, compressor, basic effects) implemented in-house — no additional crates needed. These are simple signal processing operations that don't justify a dependency.

### Latency budget

Target: < 20ms total latency (input → processed output). This is the threshold below which users can't notice audio lag during calls.

Buffer size: 256 samples at 48kHz = ~5.3ms. Processing chain overhead: ~5-10ms. OS audio system latency: ~5ms. Total: ~15-20ms. Achievable.

---

## Platform-specific virtual device implementation

### Linux (PipeWire)

**Status:** Best case. No driver, no user action, no install.

When the user starts Voice Effects:
```rust
// Spawn pw-loopback to create a virtual source device
let child = tokio::process::Command::new("pw-loopback")
    .args(&[
        "-m", "[FL FR]",
        "--capture-props", "media.class=Audio/Sink node.description=Eleutheria Microphone",
        "--playback-props", "media.class=Audio/Source node.description=Eleutheria Microphone",
    ])
    .spawn()?;
```

The virtual device "Eleutheria Microphone" appears in PipeWire/PulseAudio device lists immediately. No reboot. No permissions. No install.

CPAL then writes processed audio to the `Eleutheria Microphone` sink. PipeWire routes it to any app that selects the source.

When Voice Effects is stopped: kill the child process. Device disappears from the list.

**Fallback if PipeWire not available (pure PulseAudio systems):**
```bash
pactl load-module module-null-sink sink_name=eleutheria_mic sink_properties=device.description="Eleutheria Microphone"
pactl load-module module-loopback source=eleutheria_mic.monitor
```

Detect which is running via `pactl info` or checking for `pipewire` process.

**Fallback if neither (ALSA only, rare in 2026):** Document in the UI that Voice Effects requires PipeWire or PulseAudio. Offer a "Help me set up PipeWire" link.

### macOS

**Status:** Good. Requires bundled AudioServerPlugin, but no reboot and no user action after initial install.

The app bundles a pre-compiled AudioServerPlugin (`.driver` bundle) in the macOS `.dmg`. The app installer copies it to `/Library/Audio/Plug-Ins/HAL/` and restarts `coreaudiod` (no reboot needed). This only needs to happen once, on first install.

After installation, the plugin is always available. When Voice Effects starts, CPAL opens the virtual device. When stopped, the device remains registered (it's a persistent driver) but is silent/inactive.

**Build requirement:** The AudioServerPlugin must be compiled as a macOS-specific target. It uses CoreAudio's `AudioServerPlugIn` API (userspace, no kernel extension required since Big Sur). It's approximately 500 lines of C/Objective-C following Apple's NullAudio sample code. This is the only non-Rust platform-specific code in the feature.

**Reference implementations:** BackgroundMusic (`BGMDriver`), BlackHole, SoundPusher — all open source, all MIT or similar. We derive from these rather than starting from scratch.

**Signing requirement:** The `.driver` bundle must be signed with an Apple Developer certificate. This is part of Phase 5 distribution work (code signing is already on the roadmap).

### Windows

**Status:** Requires one-time user setup. Honest, guided, not hidden.

Windows does not provide a public API for creating virtual audio devices without a kernel driver. The only viable approaches are:

1. **Bundle a signed kernel driver in the installer.** Requires an Extended Validation (EV) code signing certificate, which costs ~$300-500/year and requires identity verification. This is the cleanest UX but most expensive. The driver would be our own, signing it would be our responsibility.

2. **Guide the user to install VB-Cable.** Free, widely trusted (millions of installs), maintained since 2012. The app detects whether it's installed, and if not, offers to download and run the installer automatically. User sees one "Install audio driver" button.

3. **Guide the user to install VB-Cable as part of the app installer.** Bundle the VB-Cable installer in the Eleutheria Windows installer. User installs Eleutheria, and during install sees: "Optional: install Eleutheria audio driver (enables Voice Effects)." This is the best UX for Windows.

**Recommendation for Phase 5:** Option 3. Bundle the VB-Cable installer. The driver install is optional during app install, and can be triggered later from within the app. VB-Cable's license permits distribution bundled with other software.

When VB-Cable is installed, CPAL sees it as a standard WASAPI device. CPAL writes processed audio to the VB-Cable input. Other apps see the VB-Cable output as a microphone.

**DECISIONS.md entry required:** Document the VB-Cable bundling decision, the license review outcome, and the fallback plan if VB-Cable's license changes.

### Android

**Status:** Good. Uses Android's AudioEffect API, no virtual device needed for most cases.

On Android, Voice Effects routes processed audio directly to the app's audio output during a call via Android's `AudioEffect` and `AudioRecord`/`AudioTrack` APIs. This is the standard mechanism for call audio processing on Android.

The virtual mic concept doesn't translate directly to Android — apps can't freely route audio between each other due to Android's security model. However, the most common use case (voice processing during calls) works via Android's built-in audio session handling. Phase 6.

---

## UI design — Voice Effects panel

The panel has three zones:

**Zone 1: Input**
- Microphone selector (default: system default mic)
- Input level meter (live, animated)
- A/B toggle: "Hear your mic without effects" (for comparison)

**Zone 2: Effects**
- Toggle switches for each effect, presented as friendly cards
- Each card has a toggle + one or two controls maximum
- "Noise reduction" card: toggle + "Strength" slider (low/medium/high)
- "Warmth" card: toggle + one slider
- Preset buttons at top: "Clean," "Broadcast," "Gaming" — these configure multiple effects at once

**Zone 3: Output**
- "Route to:" dropdown — "Virtual microphone (for calls/streams)" / "Voice recorder" / "Both"
- Output level meter (post-effects)
- Status: "Eleutheria Microphone is active" / "Not active"
- [Start] / [Stop] button

---

## Border cases

### User starts Voice Effects but another app has exclusive access to the mic
Error message: "Can't access your microphone — another app is using it exclusively." (Common with apps that use WASAPI exclusive mode on Windows.) Suggestion: close the other app or use the shared mode.

### Sample rate mismatch between real mic and virtual device
Handled by `rubato` for sample rate conversion. Transparent to the user.

### CPU usage spike from effects processing
Real-time audio processing is CPU-intensive. Effects run in a dedicated thread (not tokio async — audio processing must never be preempted by the executor). If CPU usage exceeds 80% on the processing thread, automatically reduce effect chain complexity and notify the user: "Processing reduced — too many effects for this device."

### User closes the app while Voice Effects is active
Virtual device stays registered but stops receiving audio (it goes silent). On Linux: kill the `pw-loopback` subprocess on app exit. On macOS: the driver persists (it's installed). On Windows: VB-Cable persists. All apps using the virtual mic will see silence or disconnect.

The app shows a persistent system tray notification when Voice Effects is active. Closing the window doesn't stop effects — stopping requires clicking [Stop] or using the tray menu.

### Virtual device disappears (OS restart, PipeWire restart)
Detection: CPAL reports the output device as unavailable. App shows: "Eleutheria Microphone was disconnected — restart Voice Effects." On Linux, restarting Voice Effects re-creates the loopback device.

### Two instances of the app running (shouldn't happen, but...)
The second instance detects the virtual device is already active (the first instance owns it) and refuses to start Voice Effects: "Voice Effects is already running in another window."

---

## What this is NOT

- Not a DAW or professional audio production tool
- Not a replacement for hardware audio interfaces
- Not a system-wide audio processor (only processes the mic input, not all system audio — that's a different feature and different complexity)
- Not available in Phase 5 on iOS (Tauri iOS not stable)

---

## Research dependencies (R1–R3 from Phase 4.7)

This spec can be finalized for implementation only after:
- R1 (CDP): determines whether CDP DSP is available for advanced effects (pitch shift, etc.)
- R3 (SoundThread): determines whether embedding SoundThread is viable for the audio editor which shares DSP infrastructure with Voice Effects

The basic effects (noise reduction, EQ, compressor, gate) do NOT depend on CDP or SoundThread and can be implemented immediately after Phase 5 distribution work is complete.
