use crate::github::spawn_github_fetch_thread;
use crate::installer::spawn_install_thread;
use crate::utils::format_bytes;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct MyApp {
  pub selected_folder: Option<std::path::PathBuf>,
  pub selected_file_index: usize,
  pub files: Arc<Mutex<Vec<(String, u64, String)>>>,
  pub version: Arc<Mutex<String>>,
  pub is_downloading: Arc<Mutex<bool>>,
  pub download_progress: Arc<Mutex<f32>>,
  pub progress_status: Arc<Mutex<String>>,
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
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    cc.egui_ctx.set_visuals(egui::Visuals::dark());
    let app = Self::default();
    spawn_github_fetch_thread(app.files.clone(), app.version.clone(), cc.egui_ctx.clone());
    app
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

                  spawn_install_thread(
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
