Foldean - Cross-platform Downloads Organizer

Overview
Foldean organizes your Downloads folder into clean, predictable subfolders based on file type. It runs as a safe dry-run by default and only moves files when you pass --apply.

Key features
- Cross-platform: macOS, Windows, Linux
- Safe by default: preview first, then apply
- Name collision handling: adds (1), (2), ...
- Fast: uses atomic renames; falls back to copy+remove when needed
- Extensive categories: Documents, Sheets, Slides, Images, Audio, Videos, Code, Books, Archives, Installer, Design, Others

Install locally (user-wide)
1) Open a terminal in the project root.
2) Build and install into your Cargo bin directory:
   cargo install --path .
3) Ensure Cargo bin is on PATH (zsh/macOS):
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
   On Windows (PowerShell):
   $env:Path += ";$env:USERPROFILE\\.cargo\\bin"

Run
Preview (dry run):
  foldean
Apply the changes (actually move files):
  foldean --apply
Target a different directory:
  foldean --dir /path/to/folder --apply
Include hidden files and temporary Office files:
  foldean --include-hidden --apply
Scan inside first-level subfolders (depth 1) instead of only the top level:
  foldean --depth 1 --apply

Build without installing
Debug run from source:
  cargo run --
  cargo run -- --apply
Optimized binary (kept in target/release/):
  cargo build --release
  target/release/foldean --apply

How it organizes
Examples of extension-to-folder mapping (case-insensitive):
- Documents: pdf, doc, docx, rtf, txt, md, markdown, odt, oxps
- Sheets: xls, xlsx, csv, ods
- Slides: ppt, pptx, key
- Images: jpg, jpeg, png, gif, webp, svg, bmp, tiff, heic
- Audio: mp3, wav, m4a, flac, aac, ogg
- Videos: mp4, mov, mkv, avi, webm
- Code: c, cpp, h, hpp, rs, py, js, ts, tsx, java, go, rb, sh, yaml, yml, json, toml
- Books: epub, mobi, azw, azw3, pdf
- Archives: zip, rar, 7z, tar, gz, bz2, xz
- Installer: dmg, pkg, msi, exe, deb, rpm, appimage, app
- Design: psd, ai, xd, fig, sketch
- Others: anything not matched above

Safety notes
- Default mode prints the plan and does not move files. Use --apply to perform moves.
- Hidden files (dotfiles) and Office temporary files (starting with ~$) are skipped unless you pass --include-hidden.
- If a destination name already exists, Foldean creates a unique name like file (1).ext, file (2).ext, etc.

Customization
The category mapping is defined in src/main.rs inside CATEGORY_EXTENSIONS. Edit that map to change folder names or extend the recognized file types, then rebuild/install.

Uninstall
  cargo uninstall foldean

Troubleshooting
- Command not found: ensure $HOME/.cargo/bin is on your PATH.
- Permission denied or cross-device moves: Foldean falls back to copy+remove when an atomic rename is not possible.
- To see what would happen without making changes, omit --apply.

License
This project is provided as-is without warranty.

