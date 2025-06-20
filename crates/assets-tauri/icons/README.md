# Icons Directory

This directory should contain app icons in various formats for different platforms.

To generate icons from a source image, use:

```bash
cargo tauri icon path/to/icon.png
```

Required formats:
- 32x32.png
- 128x128.png  
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)

For now, Tauri will use default icons during development.