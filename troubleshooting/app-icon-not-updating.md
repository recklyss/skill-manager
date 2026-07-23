# App icon still shows the old logo after changing it

**Symptom:** You replaced the icon files but `npm run dev` (or the dock/window) still
shows the old logo.

## Root cause

Tauri **embeds the icon into the compiled Rust binary at build time** (via the
build script that reads the `icon` array in `src-tauri/tauri.conf.json`). Editing
the PNG/ICNS files does **not** trigger a Rust rebuild, so `tauri dev` keeps
reusing the old binary — with the old icon baked in. macOS then makes it worse by
aggressively caching app icons in IconServices.

So two separate staleness layers must be cleared: the **compiled binary** and the
**macOS icon cache**.

## Fix (macOS)

Quit any running `npm run dev` first, then:

```bash
# 1. Regenerate every icon asset from a source PNG (1024x1024 recommended).
#    Use a squircle with transparent padding (~80% content) so it looks native.
npx tauri icon path/to/source-1024.png

# 2. Remove any stale build bundle that still carries the old icon.
rm -rf src-tauri/target/*/bundle

# 3. Force the Rust app crate + build script to rebuild so the new icon embeds.
cd src-tauri
touch tauri.conf.json          # makes the build script re-run
cargo clean -p skill-manager   # drops the stale compiled artifacts
cd ..

# 4. Clear the macOS icon caches (user level, no sudo).
rm -rf "$HOME/Library/Caches/com.apple.iconservices"*
find "$(getconf DARWIN_USER_CACHE_DIR)" -maxdepth 1 -name "com.apple.iconservices*" -exec rm -rf {} +

# 5. Restart Dock + Finder so they re-read icons (they auto-relaunch).
#    Get PIDs first, then kill by PID:
kill "$(pgrep -x Dock)" "$(pgrep -x Finder)"

# 6. Relaunch — first run recompiles, then shows the new icon.
npm run dev
```

If the dock **still** shows the old icon, clear the system-level cache (needs sudo):

```bash
sudo rm -rf /Library/Caches/com.apple.iconservices.store && killall Dock
```

## Notes

- The in-app / browser-tab favicon is separate: it's `frontend/public/favicon.png`
  referenced from `frontend/index.html`. Update that too for consistency.
- Making a native-looking icon: the artwork must be a rounded squircle with
  transparent margins baked in (~80% of the canvas). A full-bleed square looks
  oversized and hard-cornered next to native apps — macOS does **not** round or
  pad icons for you.
- Key takeaway: **icon changes require a Rust rebuild** (`cargo clean -p skill-manager`),
  not just regenerating the image files.
