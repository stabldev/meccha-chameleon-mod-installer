use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn spawn_github_fetch_thread(
  files: Arc<Mutex<Vec<(String, u64, String)>>>,
  version: Arc<Mutex<String>>,
  ctx: egui::Context,
) {
  std::thread::spawn(move || {
    let _ = fetch_github_release(version, files, ctx);
  });
}

fn fetch_github_release(
  version_lock: Arc<Mutex<String>>,
  files_lock: Arc<Mutex<Vec<(String, u64, String)>>>,
  ctx: egui::Context,
) -> Result<(), Box<dyn std::error::Error>> {
  let req = ureq::get(
    "https://api.github.com/repos/stabldev/meccha-chameleon-mod-installer/releases/latest",
  )
  .set("User-Agent", "meccha-chameleon-installer");

  let response = req.call()?;
  let json: serde_json::Value = response.into_json()?;

  if let Some(tag_name) = json["tag_name"].as_str() {
    *version_lock.lock().unwrap() = tag_name.to_string();
  }

  if let Some(assets) = json["assets"].as_array() {
    let mut new_files: Vec<(String, u64, String)> = assets
      .iter()
      .filter_map(|asset| {
        if let Some(content_type) = asset["content_type"].as_str() {
          if content_type == "application/x-msdownload" {
            return None;
          }
        }
        let name = asset["name"].as_str()?.to_string();
        let size = asset["size"].as_u64()?;
        let url = asset["browser_download_url"].as_str()?.to_string();
        Some((name, size, url))
      })
      .collect();
    new_files.sort_by_key(|file| file.1);
    *files_lock.lock().unwrap() = new_files;
  }

  ctx.request_repaint();
  Ok(())
}
