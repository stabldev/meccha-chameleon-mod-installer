#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 230.0]),
    ..Default::default()
  };
  eframe::run_native(
    "Meccha Chameleon Mod Installer",
    options,
    Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
  )
}

fn format_bytes(bytes: u64) -> String {
  let mb = bytes as f64 / 1_048_576.0;
  format!("{:.1} MB", mb)
}

struct MyApp {
  selected_folder: Option<std::path::PathBuf>,
  selected_file_index: usize,
  files: Arc<Mutex<Vec<(String, u64, String)>>>,
  version: Arc<Mutex<String>>,
  is_downloading: Arc<Mutex<bool>>,
  download_progress: Arc<Mutex<f32>>,
  progress_status: Arc<Mutex<String>>,
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      selected_folder: None,
      selected_file_index: 0,
      files: Arc::new(Mutex::new(Vec::new())),
      version: Arc::new(Mutex::new("Loading...".to_string())),
      is_downloading: Arc::new(Mutex::new(false)),
      download_progress: Arc::new(Mutex::new(0.0)),
      progress_status: Arc::new(Mutex::new(String::new())),
    }
  }
}

impl MyApp {
  fn new(cc: &eframe::CreationContext<'_>) -> Self {
    let app = Self::default();
    Self::spawn_github_fetch_thread(&app, cc.egui_ctx.clone());
    app
  }

  fn spawn_github_fetch_thread(app: &Self, ctx: egui::Context) {
    let files_clone = app.files.clone();
    let version_clone = app.version.clone();

    std::thread::spawn(move || {
      let _ = Self::fetch_github_release(version_clone, files_clone, ctx);
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

  fn spawn_install_thread(
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
}

impl eframe::App for MyApp {
  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    egui::Frame::NONE.inner_margin(10.0).show(ui, |ui| {
      ui.horizontal(|ui| {
        ui.label("Game Folder:");
        if let Some(path) = &self.selected_folder {
          ui.add(egui::Label::new(path.display().to_string()).truncate());
        } else {
          ui.label("Not selected");
        }
      });

      ui.add_space(5.0);

      if ui.button("Browse...").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
          self.selected_folder = Some(path);
        }
      }

      ui.add_space(10.0);

      let files = self.files.lock().unwrap().clone();
      let version = self.version.lock().unwrap().clone();
      let is_downloading = *self.is_downloading.lock().unwrap();
      let progress = *self.download_progress.lock().unwrap();
      let progress_status = self.progress_status.lock().unwrap().clone();

      ui.horizontal(|ui| {
        ui.label("Modpack File:");
        if files.is_empty() {
          ui.label("Loading...");
        } else {
          if self.selected_file_index >= files.len() {
            self.selected_file_index = 0;
          }

          ui.add_enabled_ui(!is_downloading, |ui| {
            egui::ComboBox::from_id_salt("modpack_dropdown")
              .selected_text(&files[self.selected_file_index].0)
              .show_ui(ui, |ui| {
                for (i, (name, _size, _url)) in files.iter().enumerate() {
                  ui.selectable_value(&mut self.selected_file_index, i, name);
                }
              });
          });
        }
      });

      ui.horizontal(|ui| {
        ui.label("Modpack Version:");
        ui.label(version);
      });

      ui.horizontal(|ui| {
        ui.label("Modpack Size:");
        if files.is_empty() {
          ui.label("...");
        } else {
          if self.selected_file_index < files.len() {
            ui.label(format_bytes(files[self.selected_file_index].1));
          }
        }
      });

      ui.add_space(10.0);

      ui.vertical_centered(|ui| {
        if is_downloading {
          let text = if progress_status == "Extracting..." {
            "Extracting...".to_string()
          } else {
            format!("{} {:.1}%", progress_status, progress * 100.0)
          };
          ui.add_sized(
            [ui.available_width(), 30.0],
            egui::ProgressBar::new(progress)
              .text(text)
              .corner_radius(2.0),
          );
        } else {
          let can_install = self.selected_folder.is_some() && !files.is_empty();
          ui.add_enabled_ui(can_install, |ui| {
            if ui
              .add_sized([80.0, 30.0], egui::Button::new("Install"))
              .clicked()
            {
              if let Some(folder) = &self.selected_folder {
                if let Some(file) = files.get(self.selected_file_index) {
                  let download_url = file.2.clone();
                  let target_folder = folder.clone();

                  Self::spawn_install_thread(
                    ui.ctx().clone(),
                    download_url,
                    target_folder,
                    self.is_downloading.clone(),
                    self.download_progress.clone(),
                    self.progress_status.clone(),
                  );
                }
              }
            }
          });
        }
      });

      ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.horizontal(|ui| {
          ui.spacing_mut().item_spacing.x = 0.0;
          ui.label("Made by ");
          ui.hyperlink_to("stabldev", "https://github.com/stabldev");
          ui.label(" | ");
          ui.hyperlink_to(
            "Source Code",
            "https://github.com/stabldev/meccha-chameleon-mod-installer",
          );
        });
      });
    });
  }
}
