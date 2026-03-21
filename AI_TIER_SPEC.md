# AI Tier System — Experience & Technical Spec

**Status:** Pre-implementation planning. No code until approved.
**Phase:** 5

---

## The problem

The app has AI-powered features — OCR, voice transcription, translation, smart search, document understanding. These features vary wildly in quality depending on what's available: a local Whisper model is excellent for transcription; a local Llama model on a budget laptop is not excellent for "summarize this contract."

The naive solution is "offline only." The honest problem is that offline-only means non-technical users get a noticeably worse experience than users who pair the app with a cloud model. Rather than pretending the gap doesn't exist, the right design acknowledges it and gives every user the option to close it — on their own terms.

The key insight: **most non-technical users already pay for an AI subscription.** They have Claude, ChatGPT, or Gemini. They're not getting more value from those subscriptions than they could be. The opportunity is: let them use what they're already paying for, in their local tools, without their data being sent anywhere they didn't already agree to.

---

## The three tiers, stated plainly

| Tier | What it uses | Quality | Cost | Privacy |
|------|-------------|---------|------|---------|
| **Local** | Whisper, Tesseract, Opus-MT, Ollama | Excellent for transcription + OCR. Acceptable for translation. Limited for complex reasoning. | Free | Complete |
| **Your subscription** | Any API key the user already has (Claude, GPT-4, Gemini, Mistral) | Excellent across all tasks | Their existing subscription | User controls what they send |
| **Self-hosted** | Ollama, LM Studio, any OpenAI-compatible endpoint | Depends on model + hardware | Infrastructure cost | Complete |

The app does not push users toward any tier. It presents the options honestly, lets the user choose, and respects that choice.

---

## Onboarding flow — "Your AI" setup

This happens at one of two moments:
- During first-run wizard (Phase 5 onboarding), as one step in the setup sequence
- The first time a user triggers a feature that would benefit from AI and no tier is configured

### First-run wizard step: "Make it smarter (optional)"

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Make Eleutheria smarter                                   │
│                                                             │
│   The app already works great offline — OCR, voice         │
│   transcription, and translation all run on your device.   │
│                                                             │
│   Want smarter features? Connect it to an AI you           │
│   already use. It's optional, and you can change it        │
│   anytime in Settings.                                      │
│                                                             │
│   ○  Use my device only (no AI services)                    │
│      Free · Private · Works everywhere                      │
│                                                             │
│   ○  Use my Claude / ChatGPT / Gemini subscription  ←      │
│      Better quality · Uses your existing plan              │
│      Your data goes to that service (same as using         │
│      their app directly)                                    │
│                                                             │
│   ○  Use a local AI server                                  │
│      For advanced users with Ollama or similar              │
│                                                             │
│   [Skip for now]                        [Continue →]       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Key design decisions:
- "Use my device only" is the first option and is pre-selected. No dark pattern pushing toward cloud.
- The privacy note under "subscription" is honest: "Your data goes to that service." Not alarming — just accurate.
- "Skip for now" is always available. This is not a gate.

### If "Use my Claude / ChatGPT / Gemini" is selected:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Which service do you use?                                 │
│                                                             │
│   [Claude by Anthropic]                                     │
│   [ChatGPT by OpenAI]                                       │
│   [Gemini by Google]                                        │
│   [Mistral]                                                 │
│   [Something else]                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

After selecting, say "Claude by Anthropic":

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Paste your Claude API key                                 │
│                                                             │
│   Where to find it:                                         │
│   1. Go to console.anthropic.com                           │
│   2. Click "API Keys" in the left menu                     │
│   3. Click "Create Key" → copy it here                     │
│                                                             │
│   [__________________ paste here __________________]        │
│                                                             │
│   Your key is stored only on this device.                  │
│   It's never sent to Eleutheria's servers.                 │
│   (We don't have servers.)                                  │
│                                                             │
│   [Test connection]     [Save]                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Key decisions:
- Step-by-step instructions for where to find the key. Not a link — inline steps. A non-technical user who doesn't know what an API key is can follow this.
- "We don't have servers" is a moment of honest differentiation. It's brief, parenthetical, and true.
- Test connection button gives immediate feedback before saving.

### If "Use a local AI server" is selected:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Connect to your local AI                                  │
│                                                             │
│   Server address:  [http://localhost:11434      ]           │
│                    (default for Ollama)                    │
│                                                             │
│   Model name:      [llama3.2                   ]           │
│                                                             │
│   [Test connection]     [Save]                              │
│                                                             │
│   Need help setting up Ollama?                             │
│   → ollama.ai/download                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Fallback chain

Every AI-powered action follows this chain automatically:

```
1. Does this task have a local model?
   Yes → use it (no network, free, private)
   No ↓

2. Is a cloud API key configured?
   Yes → use it (better quality, sends data to that service)
   No ↓

3. Is a self-hosted endpoint configured?
   Yes → use it
   No ↓

4. Show the user why the feature is limited and offer to configure
   "This feature works better with an AI connection.
    Set one up in Settings → Your AI."
   [Set up now]  [Use basic version]
```

"Use basic version" always exists. A user who wants offline-only should never be blocked or nagged.

---

## Which features use which tier

| Feature | Local capability | Improves with cloud/self-hosted |
|---------|-----------------|--------------------------------|
| Voice transcription | Excellent (Whisper) | Minimal improvement — local is already good |
| OCR | Good (Tesseract) | Minimal improvement — local is already good |
| Translation | Acceptable (Opus-MT) | Significant improvement for nuanced text |
| Smart search / "find that thing I saved" | Basic (FTS5 keyword) | Major improvement — natural language queries |
| Document understanding ("summarize this") | Not available locally | Requires cloud or strong local model |
| Screen context awareness | Not available locally | Requires cloud or strong local model |
| OCR + translate pipeline | Acceptable | Better translation quality |

The UI communicates this clearly per feature. Smart search shows: "Basic search active — connect an AI for natural language queries."

---

## Settings panel — "Your AI"

Always accessible, not just during onboarding.

```
Settings → Your AI

Active tier: Claude (Anthropic)  [Change]

─────────────────────────────────────────────────

Local models
  Voice transcription:   Whisper Base  [Change model]  [Download larger]
  OCR:                   Tesseract     Always local
  Translation:           Opus-MT EN↔ES, EN↔FR  [Manage packs]

─────────────────────────────────────────────────

AI service (for smart features)
  Provider:  Claude by Anthropic
  Key:       sk-ant-...••••••  [Remove]  [Test]
  Used for:  Smart search · Document summaries · Screen context
  Privacy:   Text you send to smart features goes to Anthropic.
             Your notes, clipboard, and photos never leave this
             device unless you use a smart feature on them.

─────────────────────────────────────────────────

What gets sent to Claude?
  Only the text or content you explicitly process with a smart
  feature. The app never sends your data in the background.

─────────────────────────────────────────────────

[Disconnect AI service]  [Switch to a different service]
```

Key decisions:
- "What gets sent to Claude?" section is always visible when a cloud key is configured. Privacy is not hidden in a tooltip or an FAQ link — it's on the main settings screen.
- Local models are shown separately and clearly. A user can see that OCR and transcription always stay local even with a cloud key configured.

---

## User journeys

### Journey 1: Non-technical user, already has ChatGPT
Maria pays $20/month for ChatGPT Plus. She installs Eleutheria. During onboarding she selects "Use my ChatGPT subscription." She copies her API key following the inline steps. From then on, when she uses smart search or asks the app to summarize a document, it uses GPT-4. She never thinks about this again.

What she must never see: "model name," "endpoint URL," "temperature parameter," "token limit."

### Journey 2: Privacy-focused developer
Carlos wants nothing going to the cloud. He selects "Use my device only" during onboarding and skips the AI setup. All features work — voice transcription (Whisper), OCR (Tesseract), basic translation (Opus-MT). Smart search works with keyword matching. When he opens the smart search panel, it shows: "Basic search active — upgrade to AI search in Settings." He ignores this. The app doesn't nag him again.

### Journey 3: Developer with Ollama
Daniel runs Ollama on his NAS with Llama 3.3. He selects "Use a local AI server," enters `http://192.168.1.50:11434`, model `llama3.3`. Tests connection — it works. Smart features now route to his home server. His data never leaves his network.

### Journey 4: User switches from ChatGPT to Claude
She previously set up ChatGPT. She now has a Claude subscription. Goes to Settings → Your AI → "Switch to a different service." Selects Claude, pastes her Anthropic API key, tests it. Saved. The old OpenAI key is removed.

### Journey 5: API key stops working (expired, rate limited)
Smart search returns an error. Instead of a cryptic error message, the app shows: "Couldn't reach Claude — your API key may have expired. Check it in Settings → Your AI." One tap to go there.

---

## Border cases

### User pastes a wrong/invalid API key
Test connection fails immediately. Clear message: "This key doesn't work. Check that you copied it completely and try again." The app does not save an invalid key.

### API key is correct but the account has no balance
Test connection succeeds (the key is valid) but the first actual request fails. App shows: "Claude responded with a billing error. You may need to add credits to your Anthropic account." Links to the provider's billing page.

### Cloud service is down
Request fails with a network error. App falls back to local capabilities for that request and shows a small non-blocking notification: "Claude is unreachable right now — used local model instead." Does not alarm the user.

### User is in a country where a cloud API is blocked
Same as service down — falls back gracefully. User can switch to a different provider or use local only.

### User forgets which service they configured
Settings → Your AI always shows the current configuration clearly. The provider name and a masked version of the key are always visible.

### Multiple devices with different AI configurations
Each device has its own AI configuration. Sync (Phase 6) does not sync API keys between devices — this is intentional. A key configured on Device A does not automatically appear on Device B. The user configures each device independently. This is safer (keys stay on the device where they were entered) and respects that different devices may have different needs.

### Child uses the device
Parental consideration: if a child uses a family device and the parent has a cloud API key configured, the child's queries go to that cloud service. The app doesn't have per-user accounts. The safest design: the "What gets sent to Claude?" section in Settings is visible and clear, so a parent reviewing the settings knows exactly what can be sent.

---

## Technical implementation

### Storage
API keys stored in the OS keychain (not SQLite). On Linux: `secret-service` via `keyring` crate. On macOS: Keychain. On Windows: Credential Manager. Keys never in plaintext in the database.

Rust crate: `keyring` (MIT, actively maintained). Verify Rust 1.92 + Tauri 2.x compatibility. Add DECISIONS.md entry when implemented.

### API abstraction layer
A single `AiClient` trait in Rust that all AI-powered features use. Concrete implementations: `LocalModel`, `AnthropicClient`, `OpenAiClient`, `OllamaClient`. The feature code doesn't know which client it's talking to.

```rust
pub trait AiClient: Send + Sync {
    async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<String, AiError>;
    async fn is_available(&self) -> bool;
    fn tier(&self) -> AiTier;
    fn display_name(&self) -> &str;
}

pub enum AiTier { Local, Cloud, SelfHosted }
```

### Fallback logic
Implemented in a `AiRouter` that holds the configured clients and tries them in order per the fallback chain. Features call `router.complete()` — never a specific client directly.

### Settings routes
```
GET  /api/settings/ai              → current AI configuration (masked key)
POST /api/settings/ai              → save AI configuration
POST /api/settings/ai/test         → test current configuration
DELETE /api/settings/ai            → remove AI configuration
GET  /api/settings/ai/providers    → list of supported providers with setup instructions
```
