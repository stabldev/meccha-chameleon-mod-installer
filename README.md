# Meccha Chameleon Mod Installer

A lightweight, portable Rust application built with `eframe` and `egui` for seamlessly downloading and installing game modpacks.
It automatically fetches the latest `.7z` releases from GitHub and extracts them directly into your game.

<img width="322" height="262" alt="image" src="https://github.com/user-attachments/assets/1508ef6b-e2c2-4434-9ed2-d6ad1e7660dd" />

## Download & Run

[<img src="https://img.shields.io/badge/Download_Latest_Release-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Download" />
](https://github.com/stabldev/meccha-chameleon-mod-installer/releases/latest/download/meccha_chameleon_mod_installer.exe)

> **Note:** Windows SmartScreen may flag this executable as an untrusted app because it does not have a paid code-signing certificate (unknown publisher). It is completely safe to run. You can bypass the warning by clicking "More info" and then "Run anyway".

## Usage

1. Download the `.exe` file from the link above.
2. Run `meccha_chameleon_mod_installer.exe`.
3. Click **Browse...** to select your base Game Folder.
4. Select the desired modpack from the dropdown list.
5. Click **Install** to download and automatically extract the modpack directly into the `workshop` directory.

## Modpack Versions

There are different modpack options available to choose from:
* **Minimal Maps** (`minimal-maps.7z`): Contains only lighter maps (each being <100MB in size).
* **Heavy Maps** (`heavy-maps.7z.001` & `heavy-maps.7z.002`): Contains the heavier maps (>100MB).
  > Due to file size limits, this release is split into two parts.

## Build Instructions

This project requires Rust and Cargo.

1. Clone the repository.
2. Build the optimized release binary:
   ```bash
   cargo build --release
   ```
3. The standalone executable will be located at `target/release/meccha_chameleon_mod_installer.exe`.

## Dependencies

* `eframe` / `egui` - GUI framework
* `ureq` - Blocking HTTP client for GitHub API and asset downloading
* `sevenz-rust` - Pure Rust 7zip archive extraction
* `tempfile` - Secure temporary file management
* `rfd` - Native file picker dialogs
* `serde_json` - JSON parsing for GitHub API responses
* `winres` - Windows application manifest embedding
