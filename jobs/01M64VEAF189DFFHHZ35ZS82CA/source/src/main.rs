#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SAVE_DIR: &str = "saves_tabs";
const SAVE_FILE: &str = "saves_tabs/tabs_backup.json";

// --- Цветовая палитра (Modern Dark) ---
const BG_COLOR: Color32 = Color32::from_rgb(20, 20, 24);       // Фон приложения
const PANEL_COLOR: Color32 = Color32::from_rgb(30, 30, 36);    // Панели (вкладки, низ)
const EDITOR_BG: Color32 = Color32::from_rgb(15, 15, 18);      // Фон редактора
const ACCENT: Color32 = Color32::from_rgb(138, 90, 170);       // Фиолетовый акцент
const ACCENT_HOVER: Color32 = Color32::from_rgb(160, 110, 200); // Акцент при наведении
const TEXT_COLOR: Color32 = Color32::from_rgb(220, 220, 220);  // Основной текст
const MUTED_TEXT: Color32 = Color32::from_rgb(120, 120, 130);  // Второстепенный текст

#[derive(Serialize, Deserialize, Clone)]
struct Tab {
    name: String,
    content: String,
}

struct SharedState {
    script_to_execute: String,
    last_ping: Instant,
}

fn start_server(state: Arc<Mutex<SharedState>>) {
    let server = tiny_http::Server::http("127.0.0.1:8080").unwrap();
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        match url.as_str() {
            "/get_script" => {
                let mut state_guard = state.lock().unwrap();
                state_guard.last_ping = Instant::now();
                let script = state_guard.script_to_execute.clone();
                state_guard.script_to_execute.clear();
                let _ = request.respond(tiny_http::Response::from_string(script));
            }
            "/status" => {
                let state_guard = state.lock().unwrap();
                let is_online = state_guard.last_ping.elapsed().as_secs() < 5;
                let json = if is_online { "{\"status\": \"online\"}" } else { "{\"status\": \"offline\"}" };
                let response = tiny_http::Response::from_string(json)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            _ => { let _ = request.respond(tiny_http::Response::empty(404)); }
        }
    }
}

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

        let state_clone = Arc::clone(&state);
        thread::spawn(move || { start_server(state_clone); });

        Self {
            tabs: load_tabs(),
            active_tab: 0,
            shared_state: state,
            feedback: "● Готов к работе".to_string(),
        }
    }
}

impl eframe::App for MonoscriptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- НАСТРОЙКА СТИЛЯ ---
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = BG_COLOR;
        style.visuals.window_fill = PANEL_COLOR;
        style.visuals.override_text_color = Some(TEXT_COLOR);
        
        // Настройка кнопок
        style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.weak_bg_fill = PANEL_COLOR;
        style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
        
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 55);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(45, 45, 55);
        style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
        
        style.visuals.widgets.active.bg_fill = ACCENT;
        style.visuals.widgets.active.weak_bg_fill = ACCENT;
        style.visuals.widgets.active.rounding = Rounding::same(6.0);

        // Отступы
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        ctx.set_style(style);

        ctx.request_repaint_after(Duration::from_secs(1));

        // --- ВЕРХНЯЯ ПАНЕЛЬ (ВКЛАДКИ) ---
        egui::TopBottomPanel::top("tabs_panel")
            .frame(egui::Frame::none().fill(PANEL_COLOR).inner_margin(egui::Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Логотип
                ui.label(egui::RichText::new("Ɱ").color(ACCENT).size(20.0).strong());
                ui.add_space(10.0);

                // Кнопки вкладок
                let mut tab_to_delete = None;
                for i in 0..self.tabs.len() {
                    let is_active = i == self.active_tab;
                    
                    // Делаем красивую "пилюлю" для вкладки
                    let text_color = if is_active { TEXT_COLOR } else { MUTED_TEXT };
                    let bg_color = if is_active { BG_COLOR } else { Color32::TRANSPARENT };
                    
                    let text = egui::RichText::new(&self.tabs[i].name).color(text_color);
                    
                    let btn = egui::Button::new(text)
                        .fill(bg_color)
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(0.0, 26.0)); // Высота вкладки
                        
                    if ui.add(btn).clicked() {
                        self.active_tab = i;
                    }
                    
                    // Кнопка закрытия (крестик)
                    if self.tabs.len() > 1 {
                        let close_text = egui::RichText::new("×").color(MUTED_TEXT).size(14.0);
                        if ui.add(egui::Button::new(close_text).frame(false)).clicked() {
                            tab_to_delete = Some(i);
                        }
                    }
                    ui.add_space(4.0);
                }

                if let Some(idx) = tab_to_delete {
                    self.tabs.remove(idx);
                    if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
                    save_tabs(&self.tabs);
                }

                // Добавить вкладку
                if self.tabs.len() < 8 {
                    let add_text = egui::RichText::new("+").color(MUTED_TEXT).size(16.0);
                    if ui.add(egui::Button::new(add_text).frame(false)).clicked() {
                        self.tabs.push(Tab { name: format!("Script {}", self.tabs.len() + 1), content: String::new() });
                        self.active_tab = self.tabs.len() - 1;
                        save_tabs(&self.tabs);
                    }
                }

                // Статус сервера
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_online = self.shared_state.lock().unwrap().last_ping.elapsed().as_secs() < 5;
                    let (status_text, color) = if is_online { ("● online", ACCENT) } else { ("● offline", Color32::from_rgb(100, 60, 60)) };
                    ui.label(egui::RichText::new(status_text).color(color).size(12.0));
                });
            });
        });

        // --- НИЖНЯЯ ПАНЕЛЬ (КНОПКИ) ---
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(egui::Frame::none().fill(PANEL_COLOR).inner_margin(egui::Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Главная кнопка - Execute
                let exec_btn = egui::Button::new(egui::RichText::new("▶ Execute").strong())
                    .fill(ACCENT)
                    .rounding(Rounding::same(6.0));
                if ui.add(exec_btn).clicked() {
                    self.execute_script(self.tabs[self.active_tab].content.clone());
                }

                // Второстепенные кнопки
                if ui.button("🗑 Clear").clicked() {
                    self.tabs[self.active_tab].content.clear();
                    self.feedback = "● Очищено".to_string();
                }
                if ui.button("🔄 Reset").clicked() {
                    self.execute_script("local p=game.Players.LocalPlayer.Character\nif p then p:BreakJoints() end".to_string());
                }
                if ui.button("🚪 Kick").clicked() {
                    self.execute_script("game.Players.LocalPlayer:Kick(\"Monoscript\")".to_string());
                }

                // Сообщение справа
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.feedback).color(MUTED_TEXT).size(12.0));
                });
            });
        });

        // --- ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (РЕДАКТОР) ---
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_COLOR).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
            
            // Красивая рамка вокруг текстового поля
            let frame = egui::Frame::none()
                .fill(EDITOR_BG)
                .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 45)))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::same(8.0));

            frame.show(ui, |ui| {
                // Текстовое поле
                let text_edit = egui::TextEdit::multiline(&mut self.tabs[self.active_tab].content)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20)
                    .text_color(TEXT_COLOR)
                    .frame(false); // Убираем стандартную рамку egui, так как используем свою
                
                let response = ui.add_sized(ui.available_size(), text_edit);
                
                if response.lost_focus() {
                    save_tabs(&self.tabs);
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                    self.execute_script(self.tabs[self.active_tab].content.clone());
                }
            });
        });
    }
}

impl MonoscriptApp {
    fn execute_script(&mut self, script: String) {
        if script.trim().is_empty() {
            self.feedback = "⚠ Введите код".to_string();
            return;
        }
        let mut state = self.shared_state.lock().unwrap();
        state.script_to_execute = script;
        self.feedback = "✓ Скрипт отправлен".to_string();
    }
}

fn load_tabs() -> Vec<Tab> {
    if !Path::new(SAVE_DIR).exists() {
        let _ = fs::create_dir_all(SAVE_DIR);
    }
    if let Ok(data) = fs::read_to_string(SAVE_FILE) {
        if let Ok(tabs) = serde_json::from_str::<Vec<Tab>>(&data) {
            if !tabs.is_empty() { return tabs; }
        }
    }
    vec![
        Tab { name: "Main".to_string(), content: String::new() },
        Tab { name: "Script 2".to_string(), content: String::new() },
    ]
}

fn save_tabs(tabs: &[Tab]) {
    if let Ok(json) = serde_json::to_string_pretty(tabs) {
        let _ = fs::write(SAVE_FILE, json);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 500.0])
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