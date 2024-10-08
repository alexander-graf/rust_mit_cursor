use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Match {
    trigger: String,
    replace: String,
}

#[derive(Debug, Clone)]
struct EspansoHelper {
    config_dir: PathBuf,
    selected_file: String,
    files: Vec<String>,
    new_trigger: String,
    new_replacement: String,
    matches: Vec<Match>,
    yaml_indent: String,
    filter_text: String,
    editing_index: Option<usize>,
}

impl Default for EspansoHelper {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("espanso")
            .join("match");
        let files = list_yaml_files(&config_dir);
        let selected_file = files.first().cloned().unwrap_or_default();
        let mut helper = Self {
            config_dir,
            selected_file,
            files,
            new_trigger: String::new(),
            new_replacement: String::new(),
            matches: Vec::new(),
            yaml_indent: "  ".to_string(),
            filter_text: String::new(),
            editing_index: None,
        };
        helper.load_matches();
        helper
    }
}

impl EspansoHelper {
    fn refresh(&mut self) {
        // Clear all input fields
        self.new_trigger.clear();
        self.new_replacement.clear();
        self.filter_text.clear();
        self.editing_index = None;

        // Reload the directory contents
        self.files = self.list_yaml_files();

        // If the currently selected file no longer exists, select the first available file
        if !self.files.contains(&self.selected_file) {
            self.selected_file = self.files.first().cloned().unwrap_or_default();
        }

        // Reload matches from the selected file
        self.load_matches();
    }

    fn list_yaml_files(&self) -> Vec<String> {
        fs::read_dir(&self.config_dir)
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension()?.to_str()? == "yml" {
                    Some(path.file_name()?.to_str()?.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn load_matches(&mut self) {
        let file_path = self.config_dir.join(&self.selected_file);
        self.matches = if let Ok(contents) = fs::read_to_string(file_path) {
            if let Ok(data) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                if let Some(matches) = data.get("matches").and_then(|m| m.as_sequence()) {
                    matches.iter().filter_map(|m| {
                        let trigger = m.get("trigger")?.as_str()?.to_string();
                        let replace = m.get("replace")?.as_str()?.to_string();
                        Some(Match { trigger, replace })
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
    }

    fn save_matches(&self) {
        let file_path = self.config_dir.join(&self.selected_file);
        let data = serde_yaml::to_string(&serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(vec![
            (serde_yaml::Value::String("matches".to_string()), serde_yaml::Value::Sequence(
                self.matches.iter().map(|m| serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(vec![
                    (serde_yaml::Value::String("trigger".to_string()), serde_yaml::Value::String(m.trigger.clone())),
                    (serde_yaml::Value::String("replace".to_string()), serde_yaml::Value::String(m.replace.clone())),
                ]))).collect()
            )),
        ]))).unwrap();
        fs::write(file_path, data).unwrap();
    }

    fn show_match_dialog(&mut self, match_to_edit: Option<Match>) {
        // Implementiere den Dialog zum Hinzufügen/Bearbeiten von Matches
        // Beispiel:
        if let Some(match_item) = match_to_edit {
            println!("Editing match: {:?}", match_item);
        } else {
            println!("Adding new match");
        }
    }

    fn delete_match(&mut self, index: usize) {
        // Implementiere das Löschen von Matches mit Bestätigung
        // Beispiel:
        if index < self.matches.len() {
            self.matches.remove(index);
            self.save_matches();
        }
    }

    fn filtered_matches(&self) -> Vec<Match> {
        self.matches.iter().filter(|m| {
            m.trigger.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
            m.replace.to_lowercase().contains(&self.filter_text.to_lowercase())
        }).cloned().collect()
    }

    fn add_or_update_match(&mut self) {
        if !self.new_trigger.is_empty() && !self.new_replacement.is_empty() {
            let new_match = Match {
                trigger: self.new_trigger.clone(),
                replace: self.new_replacement.clone(),
            };
            
            if let Some(index) = self.editing_index {
                if index < self.matches.len() {
                    self.matches[index] = new_match;
                }
            } else {
                self.matches.push(new_match);
            }
            
            self.new_trigger.clear();
            self.new_replacement.clear();
            self.editing_index = None;
            self.save_matches();
        }
    }

    fn open_config_folder(&self) {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(self.config_dir.to_str().unwrap())
                .spawn()
                .expect("failed to execute process");
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(self.config_dir.to_str().unwrap())
                .spawn()
                .expect("failed to execute process");
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(self.config_dir.to_str().unwrap())
                .spawn()
                .expect("failed to execute process");
        }
    }
}

impl eframe::App for EspansoHelper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut temp_self = self.clone();
        let self_rc = Rc::new(RefCell::new(&mut temp_self));
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Espanso Helper");
            
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self_rc.borrow_mut().refresh();
                }
                if ui.button("Open Config Folder").clicked() {
                    self_rc.borrow().open_config_folder();
                }
            });
            
            let selected_file = self_rc.borrow().selected_file.clone();
            let files = self_rc.borrow().files.clone();
            
            egui::ComboBox::from_label("Select YAML file")
                .selected_text(&selected_file)
                .show_ui(ui, |ui| {
                    for file in &files {
                        if ui.selectable_value(&mut self_rc.borrow_mut().selected_file, file.clone(), file).changed() {
                            self_rc.borrow_mut().load_matches();
                        }
                    }
                });
            
            ui.horizontal(|ui| {
                ui.label("Filter:");
                if ui.text_edit_singleline(&mut self_rc.borrow_mut().filter_text).changed() {
                    // Filter has changed, you might want to update the filtered matches here
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("New Trigger:");
                ui.text_edit_singleline(&mut self_rc.borrow_mut().new_trigger);
            });
            
            ui.label("New Replacement:");
            ui.text_edit_multiline(&mut self_rc.borrow_mut().new_replacement);
            
            if ui.button(if self_rc.borrow().editing_index.is_some() { "Update Match" } else { "Add Match" }).clicked() {
                self_rc.borrow_mut().add_or_update_match();
            }
            
            let filtered_matches = self_rc.borrow().filtered_matches();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, match_item) in filtered_matches.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&match_item.trigger);
                        if ui.button("Edit").clicked() {
                            let mut borrowed = self_rc.borrow_mut();
                            borrowed.new_trigger = match_item.trigger.clone();
                            borrowed.new_replacement = match_item.replace.clone();
                            borrowed.editing_index = Some(index);
                        }
                        if ui.button("Delete").clicked() {
                            self_rc.borrow_mut().delete_match(index);
                        }
                    });
                    ui.label(&match_item.replace);
                    ui.separator();
                }
            });
        });
        
        // Move the changes back to self
        *self = temp_self;
    }
}

fn list_yaml_files(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str()? == "yml" {
                Some(path.file_name()?.to_str()?.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Espanso Helper",
        options,
        Box::new(|_cc| Box::new(EspansoHelper::default())),
    )
}