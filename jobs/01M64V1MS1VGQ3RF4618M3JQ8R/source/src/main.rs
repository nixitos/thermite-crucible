#![windows_subsystem = "windows"] // Скрываем черную консоль при запуске

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SAVE_DIR: &str = "saves_tabs";
const SAVE_FILE: &str = "saves_tabs/tabs_backup.json";

// --- Структура данных вкладки ---
#[derive(Serialize, Deserialize, Clone)]
struct Tab {
    name: String,
    content: String,
}

// --- Общее состояние (для передачи между UI и сервером) ---
struct SharedState {
    script_to_execute: String,
    last_ping: Instant,
}

// --- Логика HTTP-сервера (в фоновом потоке) ---
fn start_server(state: Arc<Mutex<SharedState>>) {
    let server = tiny_http::Server::http("127.0.0.1:8080").unwrap();
    
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        
        match url.as_str() {
            "/get_script" => {
                let mut state_guard = state.lock().unwrap();
                state_guard.last_ping = Instant::now();
                
                let script = state_guard.script_to_execute.clone();
                state_guard.script_to_execute.clear(); // Очищаем после отправки
                
                let response = tiny_http::Response::from_string(script);
                let _ = request.respond(response);
            }
            "/status" => {
                let state_guard = state.lock().unwrap();
                let is_online = state_guard.last_ping.elapsed().as_secs() < 5;
                let json = if is_online { "{\"status\": \"online\"}" } else { "{\"status\": \"offline\"}" };
                let response = tiny_http::Response::from_string(json)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            _ => {
                let _ = request.respond(tiny_http::Response::empty(404));
            }
        }
    }
}

// --- Основное приложение ---
struct MonoscriptApp {
    tabs: Vec<Tab>,
    active_tab: usize,
    shared_state: Arc<Mutex<SharedState>>,
    feedback: String,
}

impl Default for MonoscriptApp {
    fn default() -> Self {
        let state = Arc::new(Mutex::new(SharedState {
            script_to_execute: String::new(),
            last_ping: Instant::now() - Duration::from_secs(10),
        }));

        // Запускаем сервер в фоне
        let state_clone = Arc::clone(&state);
        thread::spawn(move || {
            start_server(state_clone);
        });

        // Загружаем вкладки
        let tabs = load_tabs();

        Self {
            tabs,
            active_tab: 0,
            shared_state: state,
            feedback: "● готов".to_string(),
        }
    }
}

impl eframe::App for MonoscriptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Темная тема
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(10, 10, 10);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(15, 15, 15);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(25, 25, 25);
        visuals.override_text_color = Some(egui::Color32::from_rgb(192, 192, 192));
        ctx.set_visuals(visuals);

        // Обновление статуса (чтобы UI перерисовывался каждую секунду)
        ctx.request_repaint_after(Duration::from_secs(1));

        // --- Верхняя панель (Вкладки и Статус) ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Ɱ monoscript").color(egui::Color32::from_rgb(138, 90, 170)));
                
                ui.separator();

                // Кнопки вкладок
                let mut tab_to_delete = None;
                for i in 0..self.tabs.len() {
                    let is_active = i == self.active_tab;
                    let text = egui::RichText::new(&self.tabs[i].name);
                    
                    if ui.selectable_label(is_active, text).clicked() {
                        self.active_tab = i;
                    }
                    
                    // Кнопка удаления вкладки (крестик)
                    if self.tabs.len() > 1 && ui.small_button("×").clicked() {
                        tab_to_delete = Some(i);
                    }
                }

                // Добавить вкладку
                if self.tabs.len() < 8 && ui.button("+").clicked() {
                    self.tabs.push(Tab { name: format!("Script {}", self.tabs.len() + 1), content: String::new() });
                    self.active_tab = self.tabs.len() - 1;
                    save_tabs(&self.tabs);
                }

                if let Some(idx) = tab_to_delete {
                    self.tabs.remove(idx);
                    if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
                    save_tabs(&self.tabs);
                }

                // Статус справа
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_online = self.shared_state.lock().unwrap().last_ping.elapsed().as_secs() < 5;
                    let status_text = if is_online { "● online" } else { "● offline" };
                    let color = if is_online { egui::Color32::from_rgb(138, 90, 170) } else { egui::Color32::from_rgb(74, 58, 58) };
                    ui.label(egui::RichText::new(status_text).color(color));
                });
            });
        });

        // --- Нижняя панель (Кнопки) ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Execute").clicked() {
                    self.execute_script(self.tabs[self.active_tab].content.clone());
                }
                
                if ui.button("Clear").clicked() {
                    self.tabs[self.active_tab].content.clear();
                    self.feedback = "● очищено".to_string();
                }
                
                if ui.button("Reset").clicked() {
                    self.execute_script("local p=game.Players.LocalPlayer.Character\nif p then p:BreakJoints() end".to_string());
                }
                
                if ui.button("Self Kick").clicked() {
                    self.execute_script("game.Players.LocalPlayer:Kick(\"Self kick via Monoscript\")".to_string());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.feedback);
                });
            });
        });

        // --- Центральная панель (Редактор кода) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let text_edit = egui::TextEdit::multiline(&mut self.tabs[self.active_tab].content)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(20);
            
            let response = ui.add_sized(ui.available_size(), text_edit);
            
            // Сохраняем при потере фокуса (клик вне поля)
            if response.lost_focus() {
                save_tabs(&self.tabs);
            }

            // Обработка Ctrl+Enter
            if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                self.execute_script(self.tabs[self.active_tab].content.clone());
            }
        });
    }
}

impl MonoscriptApp {
    fn execute_script(&mut self, script: String) {
        if script.trim().is_empty() {
            self.feedback = "⚠ введите код".to_string();
            return;
        }
        
        let mut state = self.shared_state.lock().unwrap();
        state.script_to_execute = script;
        self.feedback = "✓ скрипт отправлен".to_string();
    }
}

// --- Функции для работы с файлами ---
fn load_tabs() -> Vec<Tab> {
    if !Path::new(SAVE_DIR).exists() {
        let _ = fs::create_dir_all(SAVE_DIR);
    }
    
    if let Ok(data) = fs::read_to_string(SAVE_FILE) {
        if let Ok(tabs) = serde_json::from_str::<Vec<Tab>>(&data) {
            if !tabs.is_empty() {
                return tabs;
            }
        }
    }
    
    // Вкладки по умолчанию, если файла нет
    vec![
        Tab { name: "Main".to_string(), content: String::new() },
        Tab { name: "Script 2".to_string(), content: String::new() },
        Tab { name: "Script 3".to_string(), content: String::new() },
    ]
}

fn save_tabs(tabs: &[Tab]) {
    if let Ok(json) = serde_json::to_string_pretty(tabs) {
        let _ = fs::write(SAVE_FILE, json);
    }
}

// --- Точка входа ---
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 500.0]) // Размер окна
            .with_min_inner_size([600.0, 400.0])
            .with_title("monoscript"),
        ..Default::default()
    };

    eframe::run_native(
        "monoscript",
        options,
        Box::new(|_cc| Box::new(MonoscriptApp::default())),
    )
}