use eframe::egui;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub fn spawn_install_thread(
  ctx: egui::Context,
  download_url: String,
  target_folder: std::path::PathBuf,
  is_downloading: Arc<Mutex<bool>>,
  download_progress: Arc<Mutex<f32>>,
  progress_status: Arc<Mutex<String>>,
) {
  *is_downloading.lock().unwrap() = true;
  *download_progress.lock().unwrap() = 0.0;
  *progress_status.lock().unwrap() = "Downloading...".to_string();

  std::thread::spawn(move || {
    let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
      let req = ureq::get(&download_url).set("User-Agent", "meccha-chameleon-installer");
      let response = req.call()?;
      let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1);

      let mut reader = response.into_reader();
      let mut temp_file = tempfile::Builder::new().suffix(".7z").tempfile()?;
      let mut buffer = [0; 65536]; // 64KB chunks
      let mut downloaded: u64 = 0;

      loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
          break;
        }

        temp_file.write_all(&buffer[..n])?;
        downloaded += n as u64;

        *download_progress.lock().unwrap() =
          (downloaded as f32 / total_size as f32).clamp(0.0, 1.0);
        ctx.request_repaint();
      }

      *progress_status.lock().unwrap() = "Extracting...".to_string();
      ctx.request_repaint();

      let temp_path = temp_file.into_temp_path();
      let workshop_folder = target_folder
        .join("Chameleon")
        .join("Binaries")
        .join("Win64")
        .join("workshop");
      std::fs::create_dir_all(&workshop_folder)?;
      sevenz_rust::decompress_file(&temp_path, &workshop_folder)?;

      Ok(())
    })();

    *is_downloading.lock().unwrap() = false;
    ctx.request_repaint();
  });
}
