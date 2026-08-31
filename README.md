# Darkstore Concierge

A desktop demo for choosing and customising a dress from a small dark-store catalogue.

![Darkstore Concierge first screen](image.png)

[Download v0.2.0 for macOS](https://github.com/amuldotexe/darkstore-chatbot/releases/download/v0.2.0/Darkstore-Concierge-v002-universal.dmg) — universal, signed, and Apple-notarized.

## Run

Requires macOS 11 or later and an OpenAI API key for recommendations. Open the DMG, move the app to Applications, and start it. The key is kept only for the current session.

## Build

```bash
pnpm install
APPLE_NOTARY_KEYCHAIN_PROFILE=darkstore-notary pnpm build:dmg:clean
```

The profile must be a local `notarytool` Keychain profile. The build clears old artifacts, creates a universal DMG, notarizes it, staples the ticket, and verifies Gatekeeper.

## Scope

- Eight synthetic dresses; local cart only.
- No account, payment, checkout, or persistent customer data.

[Specification](docs/spec-improvement-02.md) · [Journey and architecture](docs/diagrams)
