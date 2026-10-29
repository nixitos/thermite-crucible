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

// --- Точная цветовая палитра из твоего CSS ---
const BG_MAIN: Color32 = Color32::from_rgb(10, 10, 10);       // body: #0a0a0a
const BG_EDITOR: Color32 = Color32::from_rgb(6, 6, 6);        // textarea: #060606
const BG_PANEL: Color32 = Color32::from_rgb(15, 15, 15);      // button: #0f0f0f
const BG_ACTIVE_TAB: Color32 = Color32::from_rgb(26, 26, 26); // .tab-btn.active: #1a1a1a
const BORDER_COLOR: Color32 = Color32::from_rgb(26, 26, 26);  // border: #1a1a1a
const BORDER_HOVER: Color32 = Color32::from_rgb(42, 42, 42);  // border hover: #2a2a2a
const ACCENT_PURPLE: Color32 = Color32::from_rgb(138, 90, 170); // #8a5aaa
const ACCENT_PURPLE_DARK: Color32 = Color32::from_rgb(90, 58, 122); // #5a3a7a
const TEXT_MAIN: Color32 = Color32::from_rgb(192, 192, 192);  // #c0c0c0
const TEXT_MUTED: Color32 = Color32::from_rgb(102, 102, 102); // #666666
const TEXT_DANGER: Color32 = Color32::from_rgb(122, 90, 90);  // #7a5a5a

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
    feedback_is_error: bool,
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
            feedback: "● готов".to_string(),
            feedback_is_error: false,
        }
    }
}

impl eframe::App for MonoscriptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Глобальный стиль ---
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = BG_MAIN;
        style.visuals.override_text_color = Some(TEXT_MAIN);
        // Убираем отступы у панелей по умолчанию, чтобы контролировать их вручную
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        ctx.set_style(style);

        ctx.request_repaint_after(Duration::from_secs(1));

        // --- 1. ХЕДЕР (Title + Status) ---
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(BG_MAIN).inner_margin(egui::Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Логотип
                    ui.label(egui::RichText::new("Ɱ").color(ACCENT_PURPLE).size(20.0).strong());
                    ui.label(egui::RichText::new("monoscript").color(TEXT_MAIN).size(18.0));
                    
                    // Статус (прижат вправо)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_online = self.shared_state.lock().unwrap().last_ping.elapsed().as_secs() < 5;
                        let (text, color, border) = if is_online { 
                            ("● online", ACCENT_PURPLE, ACCENT_PURPLE_DARK) 
                        } else { 
                            ("● offline", Color32::from_rgb(74, 58, 58), Color32::from_rgb(42, 26, 26)) 
                        };
                        
                        // Рисуем "таблетку" статуса
                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0_f32, border))
                            .rounding(Rounding::same(12.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(text).color(color).size(11.0));
                            });
                    });
                });
                
                // Линия под хедером
                let rect = ui.max_rect();
                ui.painter().hline(rect.x_range(), rect.bottom() + 4.0, Stroke::new(1.0_f32, BORDER_COLOR));
            });

        // --- 2. ВКЛАДКИ ---
        egui::TopBottomPanel::top("tabs_panel")
            .frame(egui::Frame::none().fill(BG_MAIN).inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 4.0, bottom: 0.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0; // gap: 4px

                    let mut tab_to_delete = None;
                    for i in 0..self.tabs.len() {
                        let is_active = i == self.active_tab;
                        
                        // Красим активную вкладку
                        let bg = if is_active { BG_ACTIVE_TAB } else { Color32::TRANSPARENT };
                        let text_col = if is_active { TEXT_MAIN } else { TEXT_MUTED };
                        let border = if is_active { BORDER_HOVER } else { BORDER_COLOR };
                        
                        // Имитация вкладки (в egui нет нативных вкладок как в браузере)
                        let frame = egui::Frame::none()
                            .fill(bg)
                            .stroke(Stroke::new(1.0_f32, border))
                            .rounding(Rounding { nw: 6.0, ne: 6.0, sw: 0.0, se: 0.0 }) // Скругляем только верх
                            .inner_margin(egui::Margin { left: 10.0, right: 6.0, top: 4.0, bottom: 4.0 });
                        
                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Текст вкладки
                                let response = ui.add(egui::Label::new(egui::RichText::new(&self.tabs[i].name).color(text_col).size(12.0)).sense(egui::Sense::click()));
                                if response.clicked() && !is_active { self.active_tab = i; }
                                
                                // Кнопка закрытия (крестик)
                                if self.tabs.len() > 1 {
                                    let close = ui.add(egui::Label::new(egui::RichText::new("×").color(TEXT_MUTED).size(14.0)).sense(egui::Sense::click()));
                                    if close.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                                    if close.clicked() { tab_to_delete = Some(i); }
                                }
                            });
                        });
                    }

                    if let Some(idx) = tab_to_delete {
                        self.tabs.remove(idx);
                        if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
                        save_tabs(&self.tabs);
                    }

                    // Кнопка "+"
                    if self.tabs.len() < 8 {
                        if ui.add(egui::Button::new(egui::RichText::new("+").color(TEXT_MUTED).size(14.0)).frame(false)).clicked() {
                            self.tabs.push(Tab { name: format!("Script {}", self.tabs.len() + 1), content: String::new() });
                            self.active_tab = self.tabs.len() - 1;
                            save_tabs(&self.tabs);
                        }
                    }
                });
                
                // Линия под вкладками
                let rect = ui.max_rect();
                ui.painter().hline(rect.x_range(), rect.bottom(), Stroke::new(1.0_f32, BORDER_COLOR));
            });

        // --- 3. ФУТЕР (Сеть + Ctrl+Enter) ---
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::none().fill(BG_MAIN).inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 8.0, bottom: 8.0 }))
            .show(ctx, |ui| {
                // Линия над футером
                let rect = ui.max_rect();
                ui.painter().hline(rect.x_range(), rect.top() - 8.0, Stroke::new(1.0_f32, Color32::from_rgb(21, 21, 21)));

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("локальная сеть 127.0.0.1:8080").color(Color32::from_rgb(42, 42, 42)).size(10.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Имитация тега <kbd>
                        egui::Frame::none().fill(BG_EDITOR).stroke(Stroke::new(1.0_f32, Color32::from_rgb(21,21,21))).rounding(Rounding::same(4.0)).inner_margin(egui::Margin::symmetric(6.0, 2.0)).show(ui, |ui| {
                            ui.label(egui::RichText::new("Ctrl+Enter").color(Color32::from_rgb(58, 58, 58)).size(10.0));
                        });
                        ui.label(egui::RichText::new("отправить").color(Color32::from_rgb(42, 42, 42)).size(10.0));
                    });
                });
            });

        // --- 4. ПАНЕЛЬ КНОПОК (Execute, Clear, Reset, ...) ---
        egui::TopBottomPanel::bottom("buttons_panel")
            .frame(egui::Frame::none().fill(BG_MAIN).inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 8.0, bottom: 8.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    
                    // Главная кнопка (Execute)
                    let btn_exec = egui::Button::new(egui::RichText::new("Execute").color(TEXT_MAIN).size(12.0))
                        .fill(BG_ACTIVE_TAB)
                        .stroke(Stroke::new(1.0_f32, ACCENT_PURPLE_DARK))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn_exec).clicked() { self.execute_script(self.tabs[self.active_tab].content.clone()); }

                    // Остальные кнопки (Clear)
                    let btn_clear = egui::Button::new(egui::RichText::new("Clear").color(TEXT_MUTED).size(12.0))
                        .fill(BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, BORDER_COLOR))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn_clear).clicked() {
                        self.tabs[self.active_tab].content.clear();
                        self.set_feedback("● очищено", false);
                    }

                    // Reset (Danger)
                    let btn_reset = egui::Button::new(egui::RichText::new("Reset").color(TEXT_DANGER).size(12.0))
                        .fill(BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(31, 21, 21)))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn_reset).clicked() {
                        self.execute_script("local p=game.Players.LocalPlayer.Character\nif p then p:BreakJoints() end".to_string());
                    }

                    // Self Kick (Danger)
                    let btn_kick = egui::Button::new(egui::RichText::new("Self Kick").color(TEXT_DANGER).size(12.0))
                        .fill(BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(31, 21, 21)))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn_kick).clicked() {
                        self.execute_script("game.Players.LocalPlayer:Kick(\"Self kick via Monoscript\")".to_string());
                    }

                    // Test
                    let btn_test = egui::Button::new(egui::RichText::new("Test").color(TEXT_MUTED).size(12.0))
                        .fill(BG_PANEL)
                        .stroke(Stroke::new(1.0_f32, BORDER_COLOR))
                        .rounding(Rounding::same(6.0));
                    if ui.add(btn_test).clicked() {
                        self.execute_script("print(\"[Monoscript] Тестовый скрипт выполнен!\")\nlocal p=game.Players.LocalPlayer\nif p.Character then print(\"Персонаж найден\") end".to_string());
                    }

                    // Статус (Feedback)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (color, border) = if self.feedback_is_error { 
                            (Color32::from_rgb(122, 90, 90), Color32::from_rgb(42, 26, 26)) 
                        } else { 
                            (Color32::from_rgb(74, 74, 74), Color32::from_rgb(21, 21, 21)) 
                        };
                        
                        egui::Frame::none()
                            .fill(BG_EDITOR)
                            .stroke(Stroke::new(1.0_f32, border))
                            .rounding(Rounding::same(12.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(&self.feedback).color(color).size(11.0));
                            });
                    });
                });
            });

        // --- 5. ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (Текстовый редактор) ---
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_MAIN).inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 10.0, bottom: 10.0 }))
            .show(ctx, |ui| {
            
            // Рамка вокруг редактора
            egui::Frame::none()
                .fill(BG_EDITOR)
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(21, 21, 21)))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut self.tabs[self.active_tab].content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .text_color(Color32::from_rgb(176, 176, 176)) // textarea: #b0b0b0
                        .frame(false); // Убираем внутреннюю рамку egui
                    
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
            self.set_feedback("⚠ введите код", true);
            return;
        }
        
        // Блокируем мьютекс только на время записи скрипта
        {
            let mut state = self.shared_state.lock().unwrap();
            state.script_to_execute = script;
        } // Замок отпускается здесь
        
        // Теперь можно менять self
        self.set_feedback("✓ скрипт отправлен", false);
    }

    fn set_feedback(&mut self, text: &str, is_error: bool) {
        self.feedback = text.to_string();
        self.feedback_is_error = is_error;
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
        Tab { name: "Script 3".to_string(), content: String::new() },
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