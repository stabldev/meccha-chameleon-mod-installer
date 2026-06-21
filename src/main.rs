#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod github;
mod installer;
mod utils;

use app::MyApp;
use eframe::egui;

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
