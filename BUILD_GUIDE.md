# 🏗️ HostsPilot Build Guide

## 📦 Building Optimized Portable EXE

### Quick Build Command
```bash
npm run tauri build
```

## ✅ What's Been Configured

### 1️⃣ **Size Optimization** (Cargo.toml)
- `opt-level = "z"` - Optimize for smallest size
- `lto = true` - Link-time optimization
- `codegen-units = 1` - Better optimization
- `strip = true` - Remove debug symbols
- `panic = "abort"` - Smaller binary

**Expected size reduction:** ~9MB → ~5-6MB

### 2️⃣ **Auto Admin Rights** (build.rs + manifest)
- Embedded Windows manifest
- Automatically requests admin rights on launch
- No need to right-click → "Run as Administrator"
- UAC prompt will appear automatically

### 3️⃣ **Metadata** (tauri.conf.json)
- **Product Name:** HostsPilot
- **Version:** 1.0.0
- **Publisher:** VladoPortos
- **Copyright:** Copyright © 2025 VladoPortos
- **Category:** DeveloperTool
- **Description:** Full description included

### 4️⃣ **Portable EXE**
- NSIS installer with LZMA compression
- Portable exe available in release folder
- No installation required for portable version

## 📁 Custom Icons

### Icon Locations
Place your custom icons in: `src-tauri/icons/`

### Current Icon Files (Already Configured)
```
src-tauri/icons/
├── favicon-32x32.png          # 32x32 pixels (app icon)
├── android-icon-192x192.png   # 192x192 pixels (high res)
├── ms-icon-310x310.png        # 310x310 pixels (Windows tile)
├── apple-icon.png             # 192x192 pixels (macOS)
├── favicon.ico                # Windows icon (multi-size)
└── [many other sizes available for various platforms]
```

### Icon Files Used by Tauri

The following icons are configured in `tauri.conf.json`:

**Main Icons:**
- `favicon-32x32.png` - Small app icon (32x32)
- `android-icon-192x192.png` - Standard app icon (192x192)
- `ms-icon-310x310.png` - Large app icon (310x310)
- `apple-icon.png` - macOS app icon (192x192)
- `favicon.ico` - Windows executable icon

**Installer Icon:**
- `favicon.ico` - Used for NSIS installer

### Available Icon Sizes

Your icons folder contains icons in the following sizes:
- 16x16, 32x32, 36x36, 48x48, 57x57, 60x60, 70x70, 72x72, 76x76
- 96x96, 114x114, 120x120, 144x144, 150x150, 152x152, 180x180
- 192x192, 310x310

### Replacing Icons

To use your own icons:

1. **Generate all required sizes** using an online tool like:
   - https://www.icoconverter.com/
   - https://realfavicongenerator.net/

2. **Replace the files** in `src-tauri/icons/` with your new icons

3. **Keep the same filenames** so Tauri can find them

4. **Rebuild the app** to see your new icons:
   ```bash
   npm run tauri build
   ```

**Icon Requirements:**
- PNG: Transparent background recommended
- ICO: Multi-size (16, 32, 48, 64, 128, 256)
- Square aspect ratio
- Clear and recognizable at small sizes

## 🚀 Build Process

### 1. Clean Build
```bash
# Clean previous builds
cd src-tauri
cargo clean
cd ..

# Build
npm run tauri build
```

### 2. Output Location
After building, find your files in:
```
src-tauri/target/release/
├── hostspilot.exe              # Portable EXE (with admin rights)
└── bundle/
    └── nsis/
        └── HostsPilot_1.0.0_x64-setup.exe  # Installer
```

### 3. Distribution

**Portable Version:**
- Distribute: `hostspilot.exe`
- Size: ~5-6 MB (optimized)
- No installation needed
- Auto-requests admin rights

**Installer Version:**
- Distribute: `HostsPilot_1.0.0_x64-setup.exe`
- Installs to user's Program Files
- Creates shortcuts
- Includes uninstaller

## 🎯 File Metadata

The EXE will have the following metadata (visible in Properties):

- **File Description:** HostsPilot - Manage your hosts files
- **Product Name:** HostsPilot
- **Product Version:** 1.0.0
- **Company:** VladoPortos
- **Copyright:** Copyright © 2025 VladoPortos
- **Original Filename:** hostspilot.exe

## 🔧 Advanced Options

### Change Version
Edit `src-tauri/tauri.conf.json`:
```json
{
  "version": "1.0.0"  // Change this
}
```

### Change Compression
Edit `src-tauri/tauri.conf.json`:
```json
{
  "bundle": {
    "windows": {
      "nsis": {
        "compression": "lzma"  // Options: "lzma", "bzip2", "none"
      }
    }
  }
}
```

### Build for Different Targets
```bash
# Just NSIS installer (current config)
npm run tauri build

# Add MSI installer
# Edit tauri.conf.json: "targets": ["nsis", "msi"]
npm run tauri build
```

## 📊 Build Time

- **First build:** 5-10 minutes (downloads dependencies)
- **Subsequent builds:** 2-5 minutes (incremental)
- **Clean build:** 3-7 minutes

## ✅ Verification

After building, verify:

1. **Admin Rights:**
   - Double-click `hostspilot.exe`
   - UAC prompt should appear automatically
   - No need to right-click → Run as Administrator

2. **File Size:**
   - Should be ~5-6 MB (down from ~9 MB)

3. **Metadata:**
   - Right-click exe → Properties → Details
   - Check all metadata is correct

4. **Icon:**
   - Exe should show your custom icon
   - Check in File Explorer and taskbar

## 🐛 Troubleshooting

### Build Fails
```bash
# Clean and rebuild
cd src-tauri
cargo clean
cd ..
npm run tauri build
```

### Icon Not Showing
- Ensure all icon files exist in `src-tauri/icons/`
- Rebuild after changing icons
- Clear icon cache: restart Windows Explorer

### Admin Rights Not Working
- Check `hostspilot.exe.manifest` exists
- Verify `build.rs` includes manifest embedding
- Rebuild from scratch

### Large File Size
- Ensure `[profile.release]` is in Cargo.toml
- Use `cargo build --release` (not debug)
- Check compression is enabled in tauri.conf.json

## 📝 Notes

- The portable EXE requires WebView2 runtime (auto-installed on Windows 10/11)
- First launch may be slower (WebView2 initialization)
- Admin rights are required for modifying hosts file
- Backups are stored in `%AppData%\HostsPilot\backups`
- Profiles are stored in `%AppData%\HostsPilot\profiles`
