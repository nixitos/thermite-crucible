// =====================================================================
// LudoClient Injector v2.1 — шуточный лаунчер в стиле ludoclient.github.io
// Rust stable 1.93, eframe/egui 0.29
// «Инъекция» — симуляция: пишется лог лоадера, как на главной сайта.
// =====================================================================
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use eframe::egui;
use egui::{
    pos2, vec2, Align, Align2, Color32, FontId, Layout, Margin, Pos2, Rect, RichText, Rounding,
    Sense, Stroke, Vec2,
};

// ===== HELPERS =====
const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

// xorshift — чтобы не тянуть rand ради шуточного клиента
struct Rng(u64);
impl Rng {
    fn new() -> Self {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
        Rng(n | 1)
    }
    fn u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn f(&mut self) -> f32 { (self.u64() >> 40) as f32 / 16_777_216.0 }
    fn range(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.f() }
    fn pick<'a, T>(&mut self, s: &'a [T]) -> &'a T { &s[(self.u64() % s.len() as u64) as usize] }
}

const KEY_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
fn gen_key(rng: &mut Rng) -> String {
    let mut s = String::from("ludik://GG.");
    for _ in 0..30 { s.push(KEY_CHARS[(rng.u64() % 62) as usize] as char); }
    s
}
fn short_key(k: &str) -> String {
    if k.len() > 26 { format!("{}…{}", &k[..18], &k[k.len() - 4..]) } else { k.to_string() }
}

fn unix() -> f64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64() }
fn utc_hhmm() -> String {
    let s = unix() as u64;
    format!("{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60)
}
fn utc_date() -> String {
    // civil_from_days (Howard Hinnant)
    let z = (unix() as i64) / 86400 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{:02}.{:02}.{}", d, m, y)
}

// ===== THEMES (палитра sss.css + ещё пять гамм) =====
#[derive(Clone, Copy)]
struct Theme {
    name: &'static str,
    bg: Color32, s1: Color32, s2: Color32, s3: Color32,
    line: Color32, line2: Color32,
    text: Color32, t2: Color32, t3: Color32,
    accent: Color32, accent_dim: Color32, danger: Color32,
}
// FIX: from_white_alpha в ecolor 0.29 не const.
// from_rgba_premultiplied — const; (a,a,a,a) = тот же premultiplied-белый.
const LINE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
const LINE2: Color32 = Color32::from_rgba_premultiplied(31, 31, 31, 31);

const THEMES: [Theme; 6] = [
    Theme { name: "Графит", bg: rgb(0x0c0c0f), s1: rgb(0x131318), s2: rgb(0x191920), s3: rgb(0x212129),
        line: LINE, line2: LINE2, text: rgb(0xececef), t2: rgb(0x9b9ba6), t3: rgb(0x5e5e6a),
        accent: rgb(0x7dd3a0), accent_dim: rgb(0x4f8f6a), danger: rgb(0xd98a8a) },
    Theme { name: "Неон", bg: rgb(0x070d09), s1: rgb(0x0d1510), s2: rgb(0x121c15), s3: rgb(0x18261c),
        line: LINE, line2: LINE2, text: rgb(0xe6f5ea), t2: rgb(0x8fae97), t3: rgb(0x54695b),
        accent: rgb(0x57e389), accent_dim: rgb(0x2f9e57), danger: rgb(0xe08a8a) },
    Theme { name: "Сталь", bg: rgb(0x0b0d12), s1: rgb(0x12151d), s2: rgb(0x171b25), s3: rgb(0x1e2330),
        line: LINE, line2: LINE2, text: rgb(0xe9ecf3), t2: rgb(0x939bad), t3: rgb(0x585f70),
        accent: rgb(0x7da7d3), accent_dim: rgb(0x4a6f9e), danger: rgb(0xd98a8a) },
    Theme { name: "Прайм", bg: rgb(0x0e0c14), s1: rgb(0x15121d), s2: rgb(0x1a1724), s3: rgb(0x221e2e),
        line: LINE, line2: LINE2, text: rgb(0xefeaf7), t2: rgb(0xa298b5), t3: rgb(0x645c75),
        accent: rgb(0xb48ce8), accent_dim: rgb(0x7a5aa8), danger: rgb(0xd98a8a) },
    Theme { name: "Янтарь", bg: rgb(0x100e09), s1: rgb(0x17140e), s2: rgb(0x1d1913), s3: rgb(0x26211a),
        line: LINE, line2: LINE2, text: rgb(0xf3eee4), t2: rgb(0xb0a48d), t3: rgb(0x6e6553),
        accent: rgb(0xe0b36a), accent_dim: rgb(0xa8823f), danger: rgb(0xd98a8a) },
    Theme { name: "Рубин", bg: rgb(0x120c0d), s1: rgb(0x1a1214), s2: rgb(0x201619), s3: rgb(0x2a1c20),
        line: LINE, line2: LINE2, text: rgb(0xf5e9ea), t2: rgb(0xb5949a), t3: rgb(0x735a5f),
        accent: rgb(0xe07b8a), accent_dim: rgb(0xa84a5a), danger: rgb(0xe8a0a0) },
];

// случайный заголовок — один в один из lox.js
const TITLES: [&str; 6] = [
    "LudoClient  > Официальный сайт ",
    "LudoClient  > Шатает анти-чит ",
    "LudoClient  > КАК ЭТО ПАТЧИТЬ Б##ТЬ ",
    "LudoClient  > Дешёво и полезно ",
    "LudoClient  > Лудомания ",
    "LudoClient  > Ты просто босс на#уй ",
];

// ===== DATA =====
struct Toy {
    tex: usize, pos: Pos2, vel: Vec2,
    angle: f32, ang_vel: f32, size: f32,
    squash: f32, grabbed: bool, was_grabbed: bool,
}
struct KeyEntry { plan: String, key: String, bound: bool }
#[derive(Clone)]
struct Profile { nick: String, hwid: String, plan: String }
struct LogLine { time: String, msg: String }
struct InjectJob { lines: Vec<String>, idx: usize, next_at: f64 }

struct LudoApp {
    rng: Rng,
    last_t: f64,
    // верхняя панель
    title: String, title_since: f64, title_next: f64, maximized: bool,
    // настройки
    theme_idx: usize, smooth_anim: bool, custom_titlebar: bool,
    auto_inject: bool, servers: Vec<String>, server_input: String,
    hook_decoy: bool, new_visual: bool, xltt_trim: String, inject_cheat: bool,
    // ключи
    keys: Vec<KeyEntry>, selected_key: usize, show_keys: bool,
    // аккаунт
    nick_input: String, pass_input: String, account: Option<Profile>,
    show_unbind: bool, unbind_input: String,
    // центр
    status: &'static str, inject_job: Option<InjectJob>,
    log: Vec<LogLine>, toast: Option<(String, f64)>, bars_t: f32,
    // плюшевые игрушки
    tex_static: egui::TextureHandle, tex_neon: egui::TextureHandle,
    aspects: [f32; 2], toys: Vec<Toy>, toys_seeded: bool, next_spawn: f64, last_bounds: Rect,
}

// ===== WIDGETS =====
fn apply_style(ctx: &egui::Context, t: &Theme) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(10.0, 6.0);
    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.panel_fill = t.s1;
    v.window_fill = t.s1;
    v.faint_bg_color = t.s2;
    v.extreme_bg_color = t.bg;
    v.code_bg_color = t.bg;
    v.window_stroke = Stroke::new(1.0, t.line2);
    v.window_rounding = Rounding::same(16.0);
    v.selection.bg_fill = t.accent_dim;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.hyperlink_color = t.accent;
    v.warn_fg_color = t.danger;
    v.error_fg_color = t.danger;
    v.widgets.noninteractive.bg_fill = t.s2;
    v.widgets.noninteractive.weak_bg_fill = t.s2;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.t2);
    v.widgets.noninteractive.rounding = Rounding::same(8.0);
    v.widgets.inactive.bg_fill = t.s3;
    v.widgets.inactive.weak_bg_fill = t.s3;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.line);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.inactive.rounding = Rounding::same(8.0);
    v.widgets.hovered.bg_fill = mix(t.s3, t.text, 0.07);
    v.widgets.hovered.weak_bg_fill = mix(t.s3, t.text, 0.07);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.t3);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.hovered.rounding = Rounding::same(8.0);
    v.widgets.hovered.expansion = 0.0;
    v.widgets.active.bg_fill = mix(t.s3, t.text, 0.12);
    v.widgets.active.weak_bg_fill = mix(t.s3, t.text, 0.12);
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.t3);
    v.widgets.active.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.open.bg_fill = t.s3;
    v.widgets.open.weak_bg_fill = t.s3;
    v.widgets.open.bg_stroke = Stroke::new(1.0, t.line2);
    v.widgets.open.fg_stroke = Stroke::new(1.0, t.text);
    ctx.set_style(style);
}

// тумблер-пилюля, как на сайте
fn switch(ui: &mut egui::Ui, t: &Theme, on: &mut bool, smooth: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if resp.clicked() { *on = !*on; }
    let time = if smooth { 0.15 } else { 0.0 };
    let k = ui.ctx().animate_bool_with_time(resp.id, *on, time);
    let r = rect.shrink(1.0);
    ui.painter().rect_filled(r, 10.0, mix(t.s3, t.accent_dim, k));
    ui.painter().rect_stroke(r, 10.0, Stroke::new(1.0, mix(t.line2, t.accent, k)));
    let x = egui::lerp((r.left() + 9.0)..=(r.right() - 9.0), k);
    ui.painter().circle_filled(pos2(x, r.center().y), 7.0, mix(t.t2, t.text, k));
    if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    resp
}

fn toggle_row(ui: &mut egui::Ui, t: &Theme, label: &str, on: &mut bool, smooth: bool) -> egui::Response {
    let mut out = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(t.t2));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            out = Some(switch(ui, t, on, smooth));
        });
    });
    out.unwrap()
}

fn section_head(ui: &mut egui::Ui, t: &Theme, name: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(name).monospace().size(11.0).color(t.t3));
    ui.add_space(6.0);
}

fn ghost_button(ui: &mut egui::Ui, t: &Theme, text: &str) -> egui::Response {
    let w = ui.available_width();
    ui.add(egui::Button::new(RichText::new(text).size(13.0).color(t.text))
        .fill(t.s2).stroke(Stroke::new(1.0, t.line2)).rounding(8.0).min_size(vec2(w, 36.0)))
}

fn small_btn(ui: &mut egui::Ui, s: &str, t: &Theme) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(s).size(13.0).color(t.t2))
        .fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(6.0).min_size(vec2(30.0, 26.0)))
}

// ===== TOYS: физика плюшевых иконок =====
fn draw_toy(p: &egui::Painter, tex: &egui::TextureHandle, toy: &Toy, aspect: f32) {
    let w = toy.size * (1.0 + 0.18 * toy.squash);
    let h = (toy.size / aspect.max(0.25)) * (1.0 - 0.22 * toy.squash);
    let (s, c) = toy.angle.sin_cos();
    let corners = [(-0.5f32, -0.5f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
    let uvs = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)];
    let mut mesh = egui::Mesh::with_texture(tex.id());
    for k in 0..4 {
        let (lx, ly) = (corners[k].0 * w, corners[k].1 * h);
        let pos = pos2(toy.pos.x + lx * c - ly * s, toy.pos.y + lx * s + ly * c);
        mesh.vertices.push(egui::epaint::Vertex { pos, uv: uvs[k], color: Color32::WHITE });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    p.add(egui::Shape::mesh(mesh));
}

fn bounce_rect(toy: &mut Toy, rect: Rect) {
    let r = toy.size * 0.5;
    let cx = toy.pos.x.clamp(rect.left(), rect.right());
    let cy = toy.pos.y.clamp(rect.top(), rect.bottom());
    let d = toy.pos - pos2(cx, cy);
    let dist = d.length();
    if dist < r {
        let n = if dist > 0.001 { d / dist } else { vec2(0.0, -1.0) };
        toy.pos += n * (r - dist + 0.5);
        let vn = toy.vel.dot(n);
        if vn < 0.0 {
            if vn < -60.0 { toy.squash = (-vn / 140.0).min(1.0); }
            toy.vel -= n * (vn * 1.45);
            toy.ang_vel += (toy.vel.x * 0.002).clamp(-1.5, 1.5);
        }
    }
}

// ===== ASSETS =====
fn load_tex(ctx: &egui::Context, name: &str, fallback: Color32) -> (egui::TextureHandle, f32) {
    for path in [format!("assets/{}.png", name), format!("{}.png", name)] {
        if let Ok(img) = image::open(&path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
            return (ctx.load_texture(name, ci, egui::TextureOptions::LINEAR), w as f32 / h as f32);
        }
    }
    // заглушка-кружок, если png не нашли
    let s = 64;
    let mut img = egui::ColorImage::new([s, s], Color32::TRANSPARENT);
    for y in 0..s { for x in 0..s {
        let dx = x as f32 - 31.5; let dy = y as f32 - 31.5;
        if dx * dx + dy * dy < 28.0 * 28.0 { img.pixels[y * s + x] = fallback; }
    }}
    (ctx.load_texture(name, img, egui::TextureOptions::LINEAR), 1.0)
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(b) = std::fs::read("assets/fonts/Inter.ttf") {
        fonts.font_data.insert("inter".into(), egui::FontData::from_owned(b));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) { f.insert(0, "inter".into()); }
    }
    if let Ok(b) = std::fs::read("assets/fonts/JetBrainsMono.ttf") {
        fonts.font_data.insert("jbm".into(), egui::FontData::from_owned(b));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) { f.insert(0, "jbm".into()); }
    }
    ctx.set_fonts(fonts);
}

// ===== APP =====
impl LudoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        let mut rng = Rng::new();
        let (tex_static, a0) = load_tex(&cc.egui_ctx, "static", rgb(0x9b9ba6));
        let (tex_neon, a1) = load_tex(&cc.egui_ctx, "neon", rgb(0x7dd3a0));
        let keys = vec![
            KeyEntry { plan: "Static".into(), key: gen_key(&mut rng), bound: true },
            KeyEntry { plan: "Prime".into(), key: gen_key(&mut rng), bound: false },
        ];
        let title = TITLES[(rng.u64() % TITLES.len() as u64) as usize].to_string();
        let mut app = Self {
            rng, last_t: -1.0,
            title, title_since: 0.0, title_next: 24.0, maximized: false,
            theme_idx: 0, smooth_anim: true, custom_titlebar: true,
            auto_inject: false, servers: Vec::new(), server_input: String::new(),
            hook_decoy: false, new_visual: true, xltt_trim: "-2".into(), inject_cheat: true,
            keys, selected_key: 0, show_keys: false,
            nick_input: String::new(), pass_input: String::new(), account: None,
            show_unbind: false, unbind_input: String::new(),
            status: "idle", inject_job: None, log: Vec::new(), toast: None, bars_t: 0.0,
            tex_static, tex_neon, aspects: [a0, a1],
            toys: Vec::new(), toys_seeded: false, next_spawn: 9.0, last_bounds: Rect::NOTHING,
        };
        app.push_log("loader v2.1 ready");
        app.push_log("hwid scan ok");
        app
    }

    fn push_log(&mut self, msg: &str) {
        self.log.push(LogLine { time: utc_hhmm(), msg: msg.to_string() });
        if self.log.len() > 80 { let ex = self.log.len() - 80; self.log.drain(..ex); }
    }
    fn toast_msg(&mut self, msg: &str) { self.toast = Some((msg.to_string(), self.last_t + 2.5)); }

    fn spawn_toy(&mut self, bounds: Rect, count: usize) {
        if bounds.width() < 100.0 { return; }
        for _ in 0..count {
            let size = self.rng.range(44.0, 62.0);
            let x = self.rng.range(bounds.left() + size, bounds.right() - size);
            self.toys.push(Toy {
                tex: (self.rng.u64() % 2) as usize,
                pos: pos2(x, bounds.top() - size),
                vel: vec2(self.rng.range(-25.0, 25.0), 30.0),
                angle: self.rng.range(-0.5, 0.5),
                ang_vel: self.rng.range(-0.8, 0.8),
                size, squash: 0.0, grabbed: false, was_grabbed: false,
            });
        }
    }

    fn start_play(&mut self, now: f64) {
        if self.inject_job.is_some() { self.toast_msg("Уже инжектится…"); return; }
        let server = if self.auto_inject && !self.servers.is_empty() {
            self.rng.pick(&self.servers).clone()
        } else { "pvp.su".to_string() };
        let key4: String = self.keys.get(self.selected_key)
            .map(|k| k.key.chars().skip(11).take(4).collect())
            .unwrap_or_else(|| "a1f9".into());
        let mut lines = vec![format!("launch → {}", server)];
        if self.inject_cheat {
            self.status = "injecting";
            lines.push(format!("inject → {}", server));
            lines.push(format!("xltt-packet handshake ok · trim {}", self.xltt_trim.trim()));
            lines.push(format!("hook-decoy: {}", if self.hook_decoy { "armed" } else { "off" }));
            lines.push(format!("hwid bound · key GG.{}", key4));
            lines.push("ludik://GG.… ready".into());
        } else {
            lines.push("cheat injection skipped".into());
        }
        self.inject_job = Some(InjectJob { lines, idx: 0, next_at: now + 0.2 });
        self.spawn_toy(self.last_bounds, 2); // салют из плюшевых иконок
    }

    // ===== SIDEBAR: КЛИЕНТ =====
    fn client_section(&mut self, ui: &mut egui::Ui, t: &Theme) {
        section_head(ui, t, "КЛИЕНТ");
        toggle_row(ui, t, "Авто-инъекция", &mut self.auto_inject, self.smooth_anim);
        if self.auto_inject {
            ui.add_space(4.0);
            egui::Frame::none().fill(t.s2).rounding(8.0).inner_margin(Margin::same(10.0)).show(ui, |ui| {
                ui.label(RichText::new("Серверы для авто-инъекции").size(12.0).color(t.t2));
                ui.add_space(6.0);
                let mut remove: Option<usize> = None;
                if self.servers.is_empty() {
                    ui.label(RichText::new("Список пуст — инъекция под все серверы").size(11.0).color(t.t3));
                } else {
                    egui::Grid::new("srv_table").num_columns(2).min_col_width(0.0).spacing([8.0, 4.0]).show(ui, |ui| {
                        for (i, s) in self.servers.iter().enumerate() {
                            ui.label(RichText::new(s).monospace().size(12.0).color(t.text));
                            if ui.add(egui::Button::new(RichText::new("✕").size(11.0).color(t.t3))
                                .fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(6.0)
                                .min_size(vec2(22.0, 20.0))).clicked() { remove = Some(i); }
                            ui.end_row();
                        }
                    });
                }
                if let Some(i) = remove { self.servers.remove(i); }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let w = ui.available_width() - 40.0;
                    ui.add(egui::TextEdit::singleline(&mut self.server_input)
                        .hint_text("pvp.su").desired_width(w).font(egui::TextStyle::Monospace));
                    if ui.add(egui::Button::new(RichText::new("+").size(14.0).color(t.text))
                        .fill(t.s3).stroke(Stroke::new(1.0, t.line2)).rounding(8.0)
                        .min_size(vec2(32.0, 30.0))).clicked() {
                        let s = self.server_input.trim().to_string();
                        if !s.is_empty() && !self.servers.contains(&s) {
                            self.servers.push(s);
                            self.server_input.clear();
                        }
                    }
                });
            });
            ui.add_space(4.0);
        }
        ui.add_space(4.0);
        if ghost_button(ui, t, "Библиотека ключей").clicked() { self.show_keys = true; }
        if let Some(k) = self.keys.get(self.selected_key) {
            ui.label(RichText::new(format!("{} · {}…", k.plan, &k.key[..14.min(k.key.len())]))
                .monospace().size(11.0).color(t.t3));
        }
        ui.add_space(6.0);
        toggle_row(ui, t, "Хук-обманка анти-читов", &mut self.hook_decoy, self.smooth_anim);
        toggle_row(ui, t, "Новый визуал чит-клиента", &mut self.new_visual, self.smooth_anim)
            .on_hover_text("Никак не меняет визуал программы. Честно.");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("XLTT-обрез пакета").size(13.0).color(t.t2));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.xltt_trim)
                    .desired_width(56.0).horizontal_align(Align::RIGHT).font(egui::TextStyle::Monospace));
            });
        });
    }

    // ===== SIDEBAR: КАСТОМИЗАЦИЯ =====
    fn custom_section(&mut self, ui: &mut egui::Ui, t: &Theme, ctx: &egui::Context) {
        section_head(ui, t, "КАСТОМИЗАЦИЯ");
        ui.label(RichText::new("Тема приложения").size(13.0).color(t.t2));
        ui.add_space(4.0);
        let w = (ui.available_width() - 16.0) / 3.0;
        for row in 0..2 {
            ui.horizontal(|ui| {
                for col in 0..3 {
                    let i = row * 3 + col;
                    let th = THEMES[i];
                    let (rect, resp) = ui.allocate_exact_size(vec2(w, 30.0), Sense::click());
                    if resp.clicked() { self.theme_idx = i; }
                    let sel = i == self.theme_idx;
                    ui.painter().rect_filled(rect, 8.0, if sel { th.s3 } else { th.s2 });
                    ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, if sel { th.accent } else { th.line2 }));
                    ui.painter().circle_filled(pos2(rect.left() + 13.0, rect.center().y), 4.0, th.accent);
                    ui.painter().text(pos2(rect.left() + 23.0, rect.center().y), Align2::LEFT_CENTER,
                        th.name, FontId::proportional(11.5), if sel { th.text } else { th.t2 });
                    if resp.hovered() { ctx.set_cursor_icon(egui::CursorIcon::PointingHand); }
                }
            });
        }
        ui.add_space(6.0);
        // FIX E0503: снимок значения до мутабельного заимствования того же поля
        let smooth = self.smooth_anim;
        toggle_row(ui, t, "Плавные анимации", &mut self.smooth_anim, smooth);
        let before = self.custom_titlebar;
        toggle_row(ui, t, "Кастомная верхняя панель", &mut self.custom_titlebar, smooth);
        if before != self.custom_titlebar {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!self.custom_titlebar));
            if !self.custom_titlebar {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.clone()));
            }
        }
    }

    // ===== SIDEBAR: ПРОЧЕЕ =====
    fn other_section(&mut self, ui: &mut egui::Ui, t: &Theme) {
        section_head(ui, t, "ПРОЧЕЕ");
        if ghost_button(ui, t, "Сайт проекта").clicked() {
            let _ = open::that("https://ludoclient.github.io");
            self.toast_msg("Открываю ludoclient.github.io…");
        }
        ui.label(RichText::new("ludoclient.github.io").monospace().size(10.0).color(t.t3));
        ui.add_space(6.0);
        let w = ui.available_width();
        let btn = egui::Button::new(RichText::new("Отвязать все ключи").size(13.0).color(t.danger))
            .fill(t.s2).stroke(Stroke::new(1.0, mix(t.danger, t.line2, 0.55))).rounding(8.0)
            .min_size(vec2(w, 36.0));
        if ui.add(btn).clicked() {
            self.unbind_input.clear();
            self.show_unbind = true;
        }
    }

    // ===== SIDEBAR: АККАУНТ =====
    fn account_section(&mut self, ui: &mut egui::Ui, t: &Theme) {
        section_head(ui, t, "АККАУНТ");
        if let Some(p) = self.account.clone() {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 15.0, t.accent_dim);
                let initial = p.nick.chars().next().map(|c| c.to_uppercase().to_string())
                    .unwrap_or_else(|| "?".into());
                ui.painter().text(rect.center(), Align2::CENTER_CENTER, initial,
                    FontId::proportional(14.0), t.bg);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(RichText::new(&p.nick).size(14.0).strong().color(t.text));
                    ui.label(RichText::new(format!("{} · HWID {}", p.plan, p.hwid))
                        .monospace().size(10.0).color(t.t3));
                });
            });
            ui.add_space(8.0);
            if ghost_button(ui, t, "Выйти").clicked() {
                self.push_log("session closed");
                self.account = None;
            }
        } else {
            let w = ui.available_width();
            ui.add(egui::TextEdit::singleline(&mut self.nick_input).hint_text("Ник").desired_width(w));
            ui.add(egui::TextEdit::singleline(&mut self.pass_input).hint_text("Пароль").password(true).desired_width(w));
            let can = !self.nick_input.trim().is_empty() && !self.pass_input.is_empty();
            let btn = egui::Button::new(RichText::new("Войти").size(13.0).color(t.bg))
                .fill(t.text).stroke(Stroke::NONE).rounding(8.0).min_size(vec2(w, 36.0));
            if ui.add_enabled(can, btn).clicked() {
                let nick = self.nick_input.trim().to_string();
                let hwid = format!("{:04X}-{:04X}", self.rng.u64() % 0x10000, self.rng.u64() % 0x10000);
                let plan = self.keys.get(self.selected_key).map(|k| k.plan.clone())
                    .unwrap_or_else(|| "Static".into());
                self.push_log(&format!("profile auth ok · {}", nick));
                self.toast_msg(&format!("Добро пожаловать, {}", nick));
                self.account = Some(Profile { nick, hwid, plan });
                self.pass_input.clear();
            }
            ui.label(RichText::new("Проект шуточный — пустит любого.").size(11.0).color(t.t3));
        }
    }

    // ===== SIDEBAR: BAN-STAT (по мотивам ssss.css) =====
    fn banstat_panel(&mut self, ctx: &egui::Context, t: &Theme, dt: f32) {
        if !self.smooth_anim { self.bars_t = 1.0; }
        else { self.bars_t = (self.bars_t + dt * 1.4).min(1.0); }
        let ease = 1.0 - (1.0 - self.bars_t).powi(3);

        egui::SidePanel::right("banstat").exact_width(252.0).resizable(false)
            .frame(egui::Frame::none().fill(t.bg).inner_margin(Margin::symmetric(14.0, 18.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt("bs").auto_shrink([false, false]).show(ui, |ui| {
                    ui.label(RichText::new("Статистика банов").size(17.0).strong().color(t.text));
                    ui.label(RichText::new("по античитам · последние 30 дней").monospace().size(10.0).color(t.t3));
                    ui.add_space(12.0);

                    const DATA: [(&str, f32, f32); 4] = [
                        ("PaperPlate", 41.0, 6.4), ("NeoAC", 27.0, -2.1),
                        ("NoHack", 19.0, 0.8), ("Valcuria", 13.0, -4.0),
                    ];
                    let cw = (ui.available_width() - 8.0) / 2.0;
                    for pair in DATA.chunks(2) {
                        ui.horizontal(|ui| {
                            for (name, pct, chg) in pair {
                                ui.allocate_ui_with_layout(vec2(cw, 86.0), Layout::top_down(Align::LEFT), |ui| {
                                    egui::Frame::none().fill(t.s1).stroke(Stroke::new(1.0, t.line))
                                        .rounding(12.0).inner_margin(Margin::same(10.0)).show(ui, |ui| {
                                            ui.set_min_width(cw - 20.0);
                                            ui.spacing_mut().item_spacing.y = 2.0;
                                            ui.label(RichText::new(*name).monospace().size(10.0).color(t.t2));
                                            let lead = *name == "PaperPlate";
                                            ui.label(RichText::new(format!("{}%", pct)).size(22.0).strong()
                                                .color(if lead { t.accent } else { t.text }));
                                            let up = *chg > 0.0;
                                            ui.label(RichText::new(format!("{}{:.1}%", if up { "▲ +" } else { "▼ " }, chg))
                                                .monospace().size(10.0).color(if up { t.danger } else { t.accent }));
                                        });
                                });
                            }
                        });
                        ui.add_space(2.0);
                    }

                    ui.add_space(12.0);
                    ui.label(RichText::new("Баны по античитам").size(13.0).strong().color(t.text));
                    ui.label(RichText::new("процент забаненных за 30 дней").monospace().size(10.0).color(t.t3));
                    ui.add_space(8.0);
                    for (name, pct, _) in DATA {
                        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
                        let p = ui.painter();
                        p.text(pos2(row.left(), row.center().y), Align2::LEFT_CENTER, name,
                            FontId::monospace(10.5), t.t2);
                        let track = Rect::from_min_max(
                            pos2(row.left() + 74.0, row.center().y - 3.0),
                            pos2(row.right() - 36.0, row.center().y + 3.0));
                        p.rect_filled(track, 3.0, t.s3);
                        let w = (track.width() * (pct / 41.0) * ease).max(6.0);
                        let fill = Rect::from_min_size(track.min, vec2(w, 6.0));
                        p.rect_filled(fill, 3.0, if name == "PaperPlate" { t.accent } else { t.t3 });
                        p.text(pos2(row.right(), row.center().y), Align2::RIGHT_CENTER,
                            format!("{}%", pct), FontId::monospace(10.5), t.t2);
                        ui.add_space(6.0);
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("обновлено: {} · 04:12 UTC", utc_date()))
                        .monospace().size(10.0).color(t.t3));
                    ui.label(RichText::new("источник: телеметрия лоадера").monospace().size(10.0).color(t.t3));
                });
            });
    }

    // ===== TOYS FRAME =====
    fn toys_frame(&mut self, ui: &mut egui::Ui, bounds: Rect, obstacles: &[Rect], dt: f32, now: f64) {
        if now >= self.next_spawn && self.toys.len() < 10 {
            self.spawn_toy(bounds, 1);
            self.next_spawn = now + 8.0;
        }
        let mut cursor: Option<egui::CursorIcon> = None;
        let n = self.toys.len();

        for i in 0..n {
            let size = self.toys[i].size;
            let r = size * 0.5;
            let hit = Rect::from_center_size(self.toys[i].pos, vec2(size, size));
            let resp = ui.interact(hit, egui::Id::new("toy").with(i), Sense::drag());

            let toy = &mut self.toys[i];
            toy.was_grabbed = toy.grabbed;
            toy.grabbed = resp.dragged();

            if toy.grabbed {
                // тащим курсором, копим скорость для броска
                let d = resp.drag_delta();
                toy.pos += d;
                let inst = d / dt.max(1.0 / 240.0);
                toy.vel = toy.vel * 0.55 + inst * 0.45;
                toy.ang_vel = (toy.vel.x * 0.004).clamp(-3.0, 3.0);
                toy.pos.x = toy.pos.x.clamp(bounds.left() + r, bounds.right() - r);
                toy.pos.y = toy.pos.y.clamp(bounds.top() + r, bounds.bottom() - r);
                cursor = Some(egui::CursorIcon::Grabbing);
            } else {
                if toy.was_grabbed {
                    let sp = toy.vel.length();
                    if sp > 1500.0 { toy.vel *= 1500.0 / sp; } // бросок!
                }
                // медленное «плюшевое» падение
                toy.vel.y += 230.0 * dt;
                toy.vel *= (1.0 - 0.5 * dt).max(0.0);
                if toy.vel.y > 120.0 { toy.vel.y = 120.0; }
                toy.pos += toy.vel * dt;
                toy.angle += toy.ang_vel * dt;
                toy.ang_vel *= (1.0 - 1.2 * dt).max(0.0);

                // стены = края сайдбаров
                if toy.pos.x - r < bounds.left() {
                    toy.pos.x = bounds.left() + r;
                    if toy.vel.x < 0.0 { toy.vel.x = -toy.vel.x * 0.5; toy.ang_vel -= toy.vel.y * 0.002; }
                }
                if toy.pos.x + r > bounds.right() {
                    toy.pos.x = bounds.right() - r;
                    if toy.vel.x > 0.0 { toy.vel.x = -toy.vel.x * 0.5; toy.ang_vel += toy.vel.y * 0.002; }
                }
                if toy.pos.y - r < bounds.top() && toy.vel.y < 0.0 {
                    toy.pos.y = bounds.top() + r;
                    toy.vel.y = -toy.vel.y * 0.4;
                }
                if toy.pos.y + r > bounds.bottom() {
                    toy.pos.y = bounds.bottom() - r;
                    if toy.vel.y > 0.0 {
                        if toy.vel.y > 60.0 {
                            toy.squash = (toy.vel.y / 120.0).min(1.0);
                            toy.ang_vel += toy.vel.x * 0.003;
                        }
                        toy.vel.y = -toy.vel.y * 0.42;
                        if toy.vel.y.abs() < 28.0 { toy.vel.y = 0.0; }
                    }
                    toy.vel.x *= (1.0 - 3.0 * dt).max(0.0);
                    toy.ang_vel -= toy.angle.sin() * 1.5 * dt; // плюш качается и встаёт ровно
                }
                // отскоки от кнопок и панелей
                for o in obstacles { bounce_rect(toy, *o); }
            }
            toy.squash = (toy.squash - 3.5 * dt).max(0.0);
            if resp.hovered() && cursor.is_none() { cursor = Some(egui::CursorIcon::Grab); }
        }

        // игрушки толкают друг друга
        for i in 0..n {
            for j in (i + 1)..n {
                let min = (self.toys[i].size + self.toys[j].size) * 0.5;
                let d = self.toys[j].pos - self.toys[i].pos;
                let dist = d.length();
                if dist < min && dist > 0.001 {
                    let nrm = d / dist;
                    let pen = min - dist;
                    let (gi, gj) = (self.toys[i].grabbed, self.toys[j].grabbed);
                    let (wi, wj) = match (gi, gj) {
                        (true, true) => (0.0, 0.0), (true, false) => (0.0, 1.0),
                        (false, true) => (1.0, 0.0), _ => (0.5, 0.5),
                    };
                    self.toys[i].pos -= nrm * pen * wi;
                    self.toys[j].pos += nrm * pen * wj;
                    let rv = (self.toys[j].vel - self.toys[i].vel).dot(nrm);
                    if rv < 0.0 {
                        let imp = -rv * 0.7;
                        self.toys[i].vel -= nrm * imp * wi;
                        self.toys[j].vel += nrm * imp * wj;
                        self.toys[i].ang_vel += imp * 0.002;
                        self.toys[j].ang_vel -= imp * 0.002;
                        if rv < -80.0 { self.toys[i].squash = 0.5; self.toys[j].squash = 0.5; }
                    }
                }
            }
        }

        let painter = ui.painter().clone();
        for toy in &self.toys {
            let tex = if toy.tex == 0 { &self.tex_static } else { &self.tex_neon };
            draw_toy(&painter, tex, toy, self.aspects[toy.tex]);
        }
        if let Some(c) = cursor { ui.ctx().set_cursor_icon(c); }
    }

    // ===== WINDOW: БИБЛИОТЕКА КЛЮЧЕЙ =====
    fn keys_window(&mut self, ctx: &egui::Context, t: &Theme) {
        if !self.show_keys { return; }
        let mut open = true;
        egui::Window::new("Библиотека ключей")
            .open(&mut open).collapsible(false).resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0]).default_width(420.0)
            .frame(egui::Frame::none().fill(t.s1).stroke(Stroke::new(1.0, t.line2))
                .rounding(16.0).inner_margin(Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Выберите активный ключ").size(12.0).color(t.t2));
                ui.add_space(8.0);
                egui::ScrollArea::vertical().max_height(240.0).auto_shrink([false, false]).show(ui, |ui| {
                    for i in 0..self.keys.len() {
                        let sel = i == self.selected_key;
                        let k = &self.keys[i];
                        let fr = egui::Frame::none()
                            .fill(if sel { t.s3 } else { t.s2 }).rounding(8.0)
                            .stroke(Stroke::new(1.0, if sel { t.accent_dim } else { t.line }))
                            .inner_margin(Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_min_width(340.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(if sel { "●" } else { "○" }).size(12.0)
                                        .color(if sel { t.accent } else { t.t3 }));
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 2.0;
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&k.plan).monospace().size(11.0).color(t.accent));
                                            ui.label(RichText::new(if k.bound { "привязан" } else { "свободен" })
                                                .monospace().size(10.0).color(if k.bound { t.t3 } else { t.accent_dim }));
                                        });
                                        ui.label(RichText::new(short_key(&k.key)).monospace().size(12.0).color(t.text));
                                    });
                                });
                            });
                        let resp = ui.interact(fr.response.rect, egui::Id::new("keyrow").with(i), Sense::click());
                        if resp.clicked() { self.selected_key = i; }
                        if resp.hovered() { ctx.set_cursor_icon(egui::CursorIcon::PointingHand); }
                        ui.add_space(6.0);
                    }
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("Сгенерировать ключ").size(13.0).color(t.bg))
                        .fill(t.text).stroke(Stroke::NONE).rounding(8.0).min_size(vec2(180.0, 36.0))).clicked() {
                        let plan = if self.rng.f() < 0.5 { "Static" } else { "Prime" }.to_string();
                        self.keys.push(KeyEntry { plan, key: gen_key(&mut self.rng), bound: false });
                        self.selected_key = self.keys.len() - 1;
                        self.push_log("key generated · GG.…");
                    }
                    if ui.add(egui::Button::new(RichText::new("Копировать").size(13.0).color(t.text))
                        .fill(t.s3).stroke(Stroke::new(1.0, t.line2)).rounding(8.0).min_size(vec2(120.0, 36.0))).clicked() {
                        if let Some(k) = self.keys.get(self.selected_key) {
                            ctx.copy_text(k.key.clone());
                            self.toast_msg("Ключ скопирован");
                        }
                    }
                });
                ui.add_space(8.0);
                ui.label(RichText::new("Ключ активируется при первом входе в лаунчер и привязывается к железу.")
                    .size(11.0).color(t.t3));
            });
        self.show_keys = open;
    }

    // ===== WINDOW: ОТВЯЗАТЬ КЛЮЧИ =====
    fn unbind_window(&mut self, ctx: &egui::Context, t: &Theme) {
        if !self.show_unbind { return; }
        let acc = self.account.clone();
        let mut open = true;
        egui::Window::new("Отвязать все ключи")
            .open(&mut open).collapsible(false).resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::none().fill(t.s1).stroke(Stroke::new(1.0, t.line2))
                .rounding(16.0).inner_margin(Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.set_width(340.0);
                if let Some(p) = acc {
                    ui.label(RichText::new("Все ключи потеряют привязку к железу.").size(13.0).color(t.t2));
                    ui.label(RichText::new(format!("Введите ник «{}» для подтверждения:", p.nick))
                        .size(12.0).color(t.t3));
                    ui.add_space(8.0);
                    ui.add(egui::TextEdit::singleline(&mut self.unbind_input)
                        .hint_text(p.nick.clone()).desired_width(340.0));
                    let ok = !p.nick.trim().is_empty()
                        && self.unbind_input.trim().to_lowercase() == p.nick.trim().to_lowercase();
                    ui.add_space(10.0);
                    let btn = egui::Button::new(RichText::new("Отвязать все ключи").size(13.0).color(t.danger))
                        .fill(t.s2).stroke(Stroke::new(1.0, mix(t.danger, t.line2, 0.5)))
                        .rounding(8.0).min_size(vec2(340.0, 36.0));
                    if ui.add_enabled(ok, btn).clicked() {
                        for k in &mut self.keys { k.bound = false; }
                        self.push_log("hwid unbound · all keys");
                        self.toast_msg("Все ключи отвязаны");
                        self.show_unbind = false;
                    }
                } else {
                    ui.label(RichText::new("Сначала войдите в аккаунт — подтверждение идёт по нику.")
                        .size(13.0).color(t.t2));
                }
            });
        self.show_unbind = self.show_unbind && open;
    }
}

impl eframe::App for LudoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let dt = if self.last_t < 0.0 { 1.0 / 60.0 } else { (now - self.last_t).clamp(0.001, 0.05) } as f32;
        self.last_t = now;
        let t = THEMES[self.theme_idx];
        apply_style(ctx, &t);

        // случайный заголовок, как в lox.js (раз в 24 секунды)
        if now >= self.title_next {
            let mut idx = (self.rng.u64() % TITLES.len() as u64) as usize;
            if TITLES[idx] == self.title { idx = (idx + 1) % TITLES.len(); }
            self.title = TITLES[idx].to_string();
            self.title_since = now;
            self.title_next = now + 24.0;
            if !self.custom_titlebar {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.clone()));
            }
        }

        // прокрутка «инъекции» по строкам
        let mut emit: Option<String> = None;
        let mut done = false;
        if let Some(job) = &mut self.inject_job {
            if now >= job.next_at {
                emit = Some(job.lines[job.idx].clone());
                job.idx += 1;
                job.next_at = now + 0.6;
                if job.idx >= job.lines.len() { done = true; }
            }
        }
        if let Some(l) = emit { self.push_log(&l); }
        if done { self.inject_job = None; self.status = "ready"; }

        // ===== КАСТОМНАЯ ВЕРХНЯЯ ПАНЕЛЬ =====
        if self.custom_titlebar {
            egui::TopBottomPanel::top("titlebar").exact_height(40.0)
                .frame(egui::Frame::none().fill(t.s2).inner_margin(Margin::symmetric(12.0, 0.0)))
                .show(ctx, |ui| {
                    let bar = ui.max_rect();
                    let drag = ui.interact(bar, ui.id().with("tb_drag"), Sense::click_and_drag());
                    if drag.drag_started() { ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag); }
                    if drag.double_clicked() {
                        self.maximized = !self.maximized;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                    }
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(self.tex_neon.id(), vec2(18.0, 18.0))));
                        ui.label(RichText::new("ludoclient").monospace().size(13.0).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if small_btn(ui, "✕", &t).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                            if small_btn(ui, "▢", &t).clicked() {
                                self.maximized = !self.maximized;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                            }
                            if small_btn(ui, "—", &t).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)); }
                        });
                    });
                    // случайный лейбл по центру, с fade-in как animateTextChange
                    let a = if self.smooth_anim { ((now - self.title_since) / 0.35).clamp(0.0, 1.0) as f32 } else { 1.0 };
                    ui.painter().text(bar.center(), Align2::CENTER_CENTER, self.title.as_str(),
                        FontId::monospace(12.0), mix(Color32::TRANSPARENT, t.t2, a));
                });
        }

        // ===== ПРАВЫЙ САЙД-БАР: НАСТРОЙКИ =====
        egui::SidePanel::right("settings").exact_width(300.0).resizable(false)
            .frame(egui::Frame::none().fill(t.s1).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt("settings_scroll").auto_shrink([false, false]).show(ui, |ui| {
                    self.client_section(ui, &t);
                    ui.add_space(10.0); ui.separator();
                    self.custom_section(ui, &t, ctx);
                    ui.add_space(10.0); ui.separator();
                    self.other_section(ui, &t);
                    ui.add_space(10.0); ui.separator();
                    self.account_section(ui, &t);
                    ui.add_space(16.0);
                });
            });

        // ===== ПРАВЫЙ САЙД-БАР: BAN-STAT =====
        self.banstat_panel(ctx, &t, dt);

        // ===== ЦЕНТР =====
        egui::CentralPanel::default().frame(egui::Frame::none().fill(t.bg)).show(ctx, |ui| {
            let bounds = ui.max_rect().shrink(2.0);
            self.last_bounds = bounds;
            if !self.toys_seeded && bounds.width() > 100.0 {
                for k in 0..5 {
                    self.spawn_toy(bounds, 1);
                    if let Some(toy) = self.toys.last_mut() { toy.pos.y -= k as f32 * 110.0; }
                }
                self.toys_seeded = true;
                self.next_spawn = now + 9.0;
            }

            let mut obstacles: Vec<Rect> = Vec::new();
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space((bounds.height() * 0.13).max(20.0));
                let play = ui.add(egui::Button::new(RichText::new("Играть").size(20.0).strong().color(t.bg))
                    .fill(t.text).stroke(Stroke::NONE).rounding(12.0).min_size(vec2(240.0, 64.0)));
                if play.clicked() { self.start_play(now); }
                obstacles.push(play.rect);
                ui.add_space(14.0);
                let row = ui.horizontal(|ui| {
                    ui.label(RichText::new("Инжектировать чит").size(13.0).color(t.t2));
                    switch(ui, &t, &mut self.inject_cheat, self.smooth_anim);
                });
                obstacles.push(row.response.rect);
                ui.add_space(26.0);

                // панель лоадера — как hero-visual на главной
                let loader = egui::Frame::none().fill(t.s1).stroke(Stroke::new(1.0, t.line))
                    .rounding(16.0).inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.set_width(380.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ludoclient · loader v2.1").monospace().size(12.0).color(t.t2));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let (col, txt) = match self.status {
                                    "injecting" => (t.danger, "injecting…"),
                                    "ready" => (t.accent, "ready"),
                                    _ => (t.t3, "idle"),
                                };
                                ui.label(RichText::new(txt).monospace().size(11.0).color(col));
                            });
                        });
                        ui.add_space(4.0);
                        for ac in ["paperplate", "neoac", "nohack", "valcuria"] {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(ac).monospace().size(12.0).color(t.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new("undetected").monospace().size(11.0).color(t.accent));
                                    let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                    ui.painter().circle_filled(dot.center(), 3.0, t.accent);
                                });
                            });
                        }
                        ui.add_space(6.0);
                        egui::Frame::none().fill(t.bg).stroke(Stroke::new(1.0, t.line))
                            .rounding(8.0).inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.set_width(360.0);
                                ui.set_min_height(118.0);
                                ui.spacing_mut().item_spacing.y = 2.0;
                                let skip = self.log.len().saturating_sub(6);
                                for l in self.log.iter().skip(skip) {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&l.time).monospace().size(11.0).color(t.accent_dim));
                                        ui.label(RichText::new(&l.msg).monospace().size(11.0).color(t.t2));
                                    });
                                }
                            });
                    });
                obstacles.push(loader.response.rect);
            });

            ui.painter().text(pos2(bounds.left() + 12.0, bounds.bottom() - 8.0), Align2::LEFT_BOTTOM,
                "© 2026 LudoClient · все инструменты — для тестирования систем",
                FontId::monospace(10.0), t.t3);

            if self.toast.as_ref().map_or(false, |(_, u)| now >= *u) { self.toast = None; }
            if let Some((msg, until)) = self.toast.clone() {
                let a = ((until - now) / 0.5).clamp(0.0, 1.0) as f32;
                ui.painter().text(pos2(bounds.center().x, bounds.top() + 20.0), Align2::CENTER_CENTER,
                    msg, FontId::proportional(13.0), mix(Color32::TRANSPARENT, t.accent, a));
            }

            self.toys_frame(ui, bounds, &obstacles, dt, now);
        });

        self.keys_window(ctx, &t);
        self.unbind_window(ctx, &t);

        ctx.request_repaint_after(Duration::from_millis(16)); // физика живёт на ~60 fps
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([1080.0, 680.0])
        .with_decorations(false)
        .with_title("LudoClient");
    if let Ok(img) = image::open("assets/neon.png").or_else(|_| image::open("neon.png")) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        viewport = viewport.with_icon(egui::IconData { rgba: rgba.into_raw(), width: w, height: h });
    }
    let opts = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("LudoClient", opts, Box::new(|cc| Ok(Box::new(LudoApp::new(cc)))))
}