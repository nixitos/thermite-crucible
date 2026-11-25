// =====================================================================
// LudoClient Injector v3.0 — лаунчер в стиле ludoclient.github.io
// Rust stable 1.93, eframe/egui 0.29
// =====================================================================
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
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

struct Rng(u64);
impl Rng {
    fn new() -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
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
    fn f(&mut self) -> f32 {
        (self.u64() >> 40) as f32 / 16_777_216.0
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

const KEY_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn gen_key(rng: &mut Rng) -> String {
    let mut s = String::from("ludik://GG.");
    for _ in 0..30 {
        s.push(KEY_CHARS[(rng.u64() % 62) as usize] as char);
    }
    s
}

fn short_key(k: &str) -> String {
    if k.len() > 26 {
        format!("{}…{}", &k[..18], &k[k.len() - 4..])
    } else {
        k.to_string()
    }
}

fn unix() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn utc_hhmm() -> String {
    let s = unix() as u64;
    format!("{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60)
}

fn utc_date() -> String {
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

// ===== CONFIG: D:/configs.ludo, fallback C:/configs.ludo =====
// Данные карты сюда НИКОГДА не пишутся — только в памяти.
fn config_path() -> PathBuf {
    let d = Path::new("D:/");
    if d.exists() {
        d.join("configs.ludo")
    } else {
        Path::new("C:/").join("configs.ludo")
    }
}

// ===== THEMES =====
#[derive(Clone, Copy)]
struct Theme {
    name: &'static str,
    bg: Color32,
    s1: Color32,
    s2: Color32,
    s3: Color32,
    line: Color32,
    line2: Color32,
    text: Color32,
    t2: Color32,
    t3: Color32,
    accent: Color32,
    accent_dim: Color32,
    danger: Color32,
}

const LINE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
const LINE2: Color32 = Color32::from_rgba_premultiplied(31, 31, 31, 31);

const THEMES: [Theme; 6] = [
    Theme {
        name: "Графит",
        bg: rgb(0x0c0c0f),
        s1: rgb(0x131318),
        s2: rgb(0x191920),
        s3: rgb(0x212129),
        line: LINE,
        line2: LINE2,
        text: rgb(0xececef),
        t2: rgb(0x9b9ba6),
        t3: rgb(0x5e5e6a),
        accent: rgb(0x7dd3a0),
        accent_dim: rgb(0x4f8f6a),
        danger: rgb(0xd98a8a),
    },
    Theme {
        name: "Неон",
        bg: rgb(0x070d09),
        s1: rgb(0x0d1510),
        s2: rgb(0x121c15),
        s3: rgb(0x18261c),
        line: LINE,
        line2: LINE2,
        text: rgb(0xe6f5ea),
        t2: rgb(0x8fae97),
        t3: rgb(0x54695b),
        accent: rgb(0x57e389),
        accent_dim: rgb(0x2f9e57),
        danger: rgb(0xe08a8a),
    },
    Theme {
        name: "Сталь",
        bg: rgb(0x0b0d12),
        s1: rgb(0x12151d),
        s2: rgb(0x171b25),
        s3: rgb(0x1e2330),
        line: LINE,
        line2: LINE2,
        text: rgb(0xe9ecf3),
        t2: rgb(0x939bad),
        t3: rgb(0x585f70),
        accent: rgb(0x7da7d3),
        accent_dim: rgb(0x4a6f9e),
        danger: rgb(0xd98a8a),
    },
    Theme {
        name: "Прайм",
        bg: rgb(0x0e0c14),
        s1: rgb(0x15121d),
        s2: rgb(0x1a1724),
        s3: rgb(0x221e2e),
        line: LINE,
        line2: LINE2,
        text: rgb(0xefeaf7),
        t2: rgb(0xa298b5),
        t3: rgb(0x645c75),
        accent: rgb(0xb48ce8),
        accent_dim: rgb(0x7a5aa8),
        danger: rgb(0xd98a8a),
    },
    Theme {
        name: "Янтарь",
        bg: rgb(0x100e09),
        s1: rgb(0x17140e),
        s2: rgb(0x1d1913),
        s3: rgb(0x26211a),
        line: LINE,
        line2: LINE2,
        text: rgb(0xf3eee4),
        t2: rgb(0xb0a48d),
        t3: rgb(0x6e6553),
        accent: rgb(0xe0b36a),
        accent_dim: rgb(0xa8823f),
        danger: rgb(0xd98a8a),
    },
    Theme {
        name: "Рубин",
        bg: rgb(0x120c0d),
        s1: rgb(0x1a1214),
        s2: rgb(0x201619),
        s3: rgb(0x2a1c20),
        line: LINE,
        line2: LINE2,
        text: rgb(0xf5e9ea),
        t2: rgb(0xb5949a),
        t3: rgb(0x735a5f),
        accent: rgb(0xe07b8a),
        accent_dim: rgb(0xa84a5a),
        danger: rgb(0xe8a0a0),
    },
];

const TITLES: [&str; 6] = [
    "LudoClient  > Официальный сайт ",
    "LudoClient  > Шатает анти-чит ",
    "LudoClient  > КАК ЭТО ПАТЧИТЬ Б##ТЬ ",
    "LudoClient  > Дешёво и полезно ",
    "LudoClient  > Лудомания ",
    "LudoClient  > Ты просто босс на#уй ",
];

const FAKE_CLIENTS: [&str; 5] = [
    "Feather Client",
    "A-Client",
    "E-Client",
    "Spire Visuals",
    "Original Client",
];

const HIDE_BYTECODE: [&str; 4] = [
    "In game assets",
    "In explorer.exe",
    "In fake chrome",
    "In virtual headphone",
];

const PARTICLE_KINDS: [&str; 4] = ["Снежинки", "Звёздочки", "Иконки чита", "Все варианты"];

const TITLEBAR_H: f32 = 32.0;

// ===== DATA =====
struct Toy {
    tex: usize,
    pos: Pos2,
    vel: Vec2,
    angle: f32,
    ang_vel: f32,
    size: f32,
    squash: f32,
    grabbed: bool,
    was_grabbed: bool,
}

struct Particle {
    kind: usize,
    base_x: f32,
    pos: Pos2,
    vy: f32,
    sway_amp: f32,
    sway_freq: f32,
    phase: f32,
    size: f32,
    angle: f32,
    ang_vel: f32,
    alpha: u8,
}

struct KeyEntry {
    plan: String,
    key: String,
    bound: bool,
}

#[derive(Clone)]
struct Profile {
    nick: String,
    hwid: String,
    plan: String,
}

struct LogLine {
    time: String,
    msg: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Modal {
    Keys,
    Buy,
    Auth,
    Unbind,
    PlayError,
}

struct LudoApp {
    rng: Rng,
    last_t: f64,
    title: String,
    title_since: f64,
    title_next: f64,
    maximized: bool,
    theme_idx: usize,
    smooth_anim: bool,
    custom_titlebar: bool,
    deco_applied: bool,
    auto_inject: bool,
    servers: Vec<String>,
    server_input: String,
    hook_decoy: bool,
    new_visual: bool,
    xltt_trim: String,
    inject_cheat: bool,
    deep_open: bool,
    ring_upper: bool,
    fake_client: bool,
    fake_client_idx: usize,
    anti_spy: bool,
    broken_kick: bool,
    hide_bytecode: bool,
    hide_bytecode_idx: usize,
    bg_particles: bool,
    bg_particle_kind: usize,
    last_particle_kind: usize,
    particles: Vec<Particle>,
    particles_seeded: bool,
    keys: Vec<KeyEntry>,
    selected_key: usize,
    key_menu: Option<(usize, Pos2)>,
    nick_input: String,
    pass_input: String,
    account: Option<Profile>,
    unbind_input: String,
    modal: Option<Modal>,
    buy_plan: usize,
    buy_period: usize,
    buy_until: f64,
    buy_dd_open: bool,
    buy_error: String,
    buy_result: Option<String>,
    copy_fb_until: f64,
    card_name: String,
    card_number: String,
    card_expiry: String,
    card_cvv: String,
    log: Vec<LogLine>,
    toast: Option<(String, f64)>,
    bars_t: f32,
    tex_static: egui::TextureHandle,
    tex_neon: egui::TextureHandle,
    aspects: [f32; 2],
    toys: Vec<Toy>,
    toys_seeded: bool,
    next_spawn: f64,
    last_bounds: Rect,
    cfg: PathBuf,
    last_sig: String,
    dirty: bool,
    last_save: f64,
}

// ===== WIDGETS =====
fn apply_style(ctx: &egui::Context, t: &Theme) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 8.0);
    style.spacing.scroll.bar_width = 8.0;
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

fn switch(ui: &mut egui::Ui, t: &Theme, on: &mut bool, smooth: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let time = if smooth { 0.15 } else { 0.0 };
    let k = ui.ctx().animate_bool_with_time(resp.id, *on, time);
    let r = rect.shrink(1.0);
    ui.painter().rect_filled(r, 10.0, mix(t.s3, t.accent_dim, k));
    ui.painter()
        .rect_stroke(r, 10.0, Stroke::new(1.0, mix(t.line2, t.accent, k)));
    let x = egui::lerp((r.left() + 9.0)..=(r.right() - 9.0), k);
    ui.painter()
        .circle_filled(pos2(x, r.center().y), 7.0, mix(t.t2, t.text, k));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn toggle_row(
    ui: &mut egui::Ui,
    t: &Theme,
    label: &str,
    on: &mut bool,
    smooth: bool,
) -> egui::Response {
    let mut out = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(t.t2));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            out = Some(switch(ui, t, on, smooth));
        });
    });
    out.unwrap()
}

fn section_head(ui: &mut egui::Ui, t: &Theme, name: &str, top: f32) {
    ui.add_space(top);
    ui.label(RichText::new(name).monospace().size(11.0).color(t.t3));
    ui.add_space(8.0);
}

fn ghost_button(ui: &mut egui::Ui, t: &Theme, text: &str) -> egui::Response {
    let w = ui.available_width();
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(t.text))
            .fill(t.s2)
            .stroke(Stroke::new(1.0, t.line2))
            .rounding(8.0)
            .min_size(vec2(w, 36.0)),
    )
}

fn field_label(ui: &mut egui::Ui, t: &Theme, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(t.t2));
    ui.add_space(4.0);
}

// caption-кнопки Windows 10: 46x32, иконки линиями, close-hover красный
fn caption_btn(ui: &mut egui::Ui, t: &Theme, kind: u8, maximized: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(46.0, TITLEBAR_H), Sense::click());
    let hover = resp.hovered();
    let bg = if kind == 2 {
        if hover {
            rgb(0xe81123)
        } else {
            Color32::TRANSPARENT
        }
    } else if hover {
        Color32::from_white_alpha(20)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 0.0, bg);
    let c = rect.center();
    let col = if kind == 2 && hover {
        Color32::WHITE
    } else {
        t.t2
    };
    let st = Stroke::new(1.0, col);
    let p = ui.painter();
    match kind {
        0 => {
            p.line_segment([pos2(c.x - 5.0, c.y), pos2(c.x + 5.0, c.y)], st);
        }
        1 => {
            if maximized {
                p.rect_stroke(
                    Rect::from_center_size(pos2(c.x + 1.5, c.y - 1.5), vec2(8.0, 8.0)),
                    0.0,
                    st,
                );
                p.rect_stroke(
                    Rect::from_center_size(pos2(c.x - 1.5, c.y + 1.5), vec2(8.0, 8.0)),
                    0.0,
                    st,
                );
            } else {
                p.rect_stroke(Rect::from_center_size(c, vec2(10.0, 10.0)), 0.0, st);
            }
        }
        _ => {
            p.line_segment([pos2(c.x - 5.0, c.y - 5.0), pos2(c.x + 5.0, c.y + 5.0)], st);
            p.line_segment([pos2(c.x - 5.0, c.y + 5.0), pos2(c.x + 5.0, c.y - 5.0)], st);
        }
    }
    resp.clicked()
}

// ===== RESIZE =====
fn resize_grip(
    ctx: &egui::Context,
    id: &str,
    rect: Rect,
    dir: egui::ResizeDirection,
    cur: egui::CursorIcon,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .movable(false)
        .interactable(true)
        .show(ctx, |ui| {
            let resp = ui.allocate_exact_size(rect.size(), Sense::drag()).1;
            if resp.hovered() {
                ctx.set_cursor_icon(cur);
            }
            if resp.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
            }
        });
}

fn resize_grips(ctx: &egui::Context) {
    let s = ctx.screen_rect();
    let e = 4.0;
    let c = 8.0;

    resize_grip(
        ctx,
        "rs_n",
        Rect::from_min_max(pos2(s.left() + c, s.top()), pos2(s.right() - c, s.top() + e)),
        egui::ResizeDirection::North,
        egui::CursorIcon::ResizeVertical,
    );
    resize_grip(
        ctx,
        "rs_s",
        Rect::from_min_max(
            pos2(s.left() + c, s.bottom() - e),
            pos2(s.right() - c, s.bottom()),
        ),
        egui::ResizeDirection::South,
        egui::CursorIcon::ResizeVertical,
    );
    resize_grip(
        ctx,
        "rs_w",
        Rect::from_min_max(pos2(s.left(), s.top() + c), pos2(s.left() + e, s.bottom() - c)),
        egui::ResizeDirection::West,
        egui::CursorIcon::ResizeHorizontal,
    );
    resize_grip(
        ctx,
        "rs_e",
        Rect::from_min_max(
            pos2(s.right() - e, s.top() + c),
            pos2(s.right(), s.bottom() - c),
        ),
        egui::ResizeDirection::East,
        egui::CursorIcon::ResizeHorizontal,
    );
    resize_grip(
        ctx,
        "rs_nw",
        Rect::from_min_max(s.min, pos2(s.left() + c, s.top() + c)),
        egui::ResizeDirection::NorthWest,
        egui::CursorIcon::ResizeNwSe,
    );
    resize_grip(
        ctx,
        "rs_ne",
        Rect::from_min_max(pos2(s.right() - c, s.top()), pos2(s.right(), s.top() + c)),
        egui::ResizeDirection::NorthEast,
        egui::CursorIcon::ResizeNeSw,
    );
    resize_grip(
        ctx,
        "rs_sw",
        Rect::from_min_max(pos2(s.left(), s.bottom() - c), pos2(s.left() + c, s.bottom())),
        egui::ResizeDirection::SouthWest,
        egui::CursorIcon::ResizeNeSw,
    );
    resize_grip(
        ctx,
        "rs_se",
        Rect::from_min_max(pos2(s.right() - c, s.bottom() - c), s.max),
        egui::ResizeDirection::SouthEast,
        egui::CursorIcon::ResizeNwSe,
    );
}

// ===== PARTICLES (задний план) =====
fn pick_kind(want: usize, rng: &mut Rng) -> usize {
    match want {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => (rng.u64() % 3) as usize,
    }
}

fn draw_snow(p: &egui::Painter, c: Pos2, r: f32, angle: f32, col: Color32) {
    let st = Stroke::new(1.0, col);
    for k in 0..3 {
        let a = angle + k as f32 * std::f32::consts::FRAC_PI_3;
        let (s, co) = a.sin_cos();
        p.line_segment(
            [pos2(c.x - co * r, c.y - s * r), pos2(c.x + co * r, c.y + s * r)],
            st,
        );
    }
}

fn draw_star(p: &egui::Painter, c: Pos2, r: f32, angle: f32, col: Color32) {
    let mut pts = Vec::with_capacity(10);
    for k in 0..10 {
        let rr = if k % 2 == 0 { r } else { r * 0.45 };
        let a = angle + k as f32 * std::f32::consts::FRAC_PI_5;
        pts.push(pos2(c.x + a.sin() * rr, c.y - a.cos() * rr));
    }
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.0, col)));
}

fn draw_particle_icon(
    p: &egui::Painter,
    tex: &egui::TextureHandle,
    c: Pos2,
    size: f32,
    angle: f32,
    col: Color32,
    aspect: f32,
) {
    let w = size;
    let h = size / aspect.max(0.25);
    let (s, co) = angle.sin_cos();
    let corners = [(-0.5f32, -0.5f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
    let uvs = [
        pos2(0.0, 0.0),
        pos2(1.0, 0.0),
        pos2(1.0, 1.0),
        pos2(0.0, 1.0),
    ];
    let mut mesh = egui::Mesh::with_texture(tex.id());
    for k in 0..4 {
        let (lx, ly) = (corners[k].0 * w, corners[k].1 * h);
        let pos = pos2(c.x + lx * co - ly * s, c.y + lx * s + ly * co);
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: uvs[k],
            color: col,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    p.add(egui::Shape::mesh(mesh));
}

// ===== TOYS =====
fn rounded_rect_pts(center: Pos2, w: f32, h: f32, r: f32, angle: f32) -> Vec<Pos2> {
    let hw = (w * 0.5 - r).max(0.0);
    let hh = (h * 0.5 - r).max(0.0);
    let (s, c) = angle.sin_cos();
    let corners = [
        (hw, -hh, -std::f32::consts::FRAC_PI_2),
        (hw, hh, 0.0),
        (-hw, hh, std::f32::consts::FRAC_PI_2),
        (-hw, -hh, std::f32::consts::PI),
    ];
    let mut pts = Vec::with_capacity(20);
    for (cx, cy, a0) in corners {
        for k in 0..=4 {
            let a = a0 + (k as f32) * (std::f32::consts::FRAC_PI_2 / 4.0);
            let lx = cx + r * a.cos();
            let ly = cy + r * a.sin();
            pts.push(pos2(center.x + lx * c - ly * s, center.y + lx * s + ly * c));
        }
    }
    pts
}

fn draw_toy(p: &egui::Painter, tex: &egui::TextureHandle, toy: &Toy, aspect: f32) {
    let w = toy.size * (1.0 + 0.18 * toy.squash);
    let h = (toy.size / aspect.max(0.25)) * (1.0 - 0.22 * toy.squash);
    let pts = rounded_rect_pts(toy.pos, w + 12.0, h + 12.0, 10.0, toy.angle);
    p.add(egui::Shape::convex_polygon(
        pts,
        Color32::from_white_alpha(51),
        Stroke::new(1.0, Color32::from_white_alpha(38)),
    ));
    let (s, c) = toy.angle.sin_cos();
    let corners = [(-0.5f32, -0.5f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
    let uvs = [
        pos2(0.0, 0.0),
        pos2(1.0, 0.0),
        pos2(1.0, 1.0),
        pos2(0.0, 1.0),
    ];
    let mut mesh = egui::Mesh::with_texture(tex.id());
    for k in 0..4 {
        let (lx, ly) = (corners[k].0 * w, corners[k].1 * h);
        let pos = pos2(toy.pos.x + lx * c - ly * s, toy.pos.y + lx * s + ly * c);
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: uvs[k],
            color: Color32::WHITE,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    p.add(egui::Shape::mesh(mesh));
}

fn bounce_rect(toy: &mut Toy, rect: Rect) {
    let r = toy.size * 0.5 + 6.0;
    let cx = toy.pos.x.clamp(rect.left(), rect.right());
    let cy = toy.pos.y.clamp(rect.top(), rect.bottom());
    let d = toy.pos - pos2(cx, cy);
    let dist = d.length();
    if dist < r {
        let n = if dist > 0.001 { d / dist } else { vec2(0.0, -1.0) };
        toy.pos += n * (r - dist + 0.5);
        let vn = toy.vel.dot(n);
        if vn < 0.0 {
            if vn < -60.0 {
                toy.squash = (-vn / 140.0).min(1.0);
            }
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
            let ci =
                egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
            return (
                ctx.load_texture(name, ci, egui::TextureOptions::LINEAR),
                w as f32 / h as f32,
            );
        }
    }
    let s = 64;
    let mut img = egui::ColorImage::new([s, s], Color32::TRANSPARENT);
    for y in 0..s {
        for x in 0..s {
            let dx = x as f32 - 31.5;
            let dy = y as f32 - 31.5;
            if dx * dx + dy * dy < 28.0 * 28.0 {
                img.pixels[y * s + x] = fallback;
            }
        }
    }
    (ctx.load_texture(name, img, egui::TextureOptions::LINEAR), 1.0)
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(b) = std::fs::read("assets/fonts/Inter.ttf") {
        fonts.font_data.insert("inter".into(), egui::FontData::from_owned(b));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            f.insert(0, "inter".into());
        }
    }
    if let Ok(b) = std::fs::read("assets/fonts/JetBrainsMono.ttf") {
        fonts.font_data.insert("jbm".into(), egui::FontData::from_owned(b));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            f.insert(0, "jbm".into());
        }
    }
    ctx.set_fonts(fonts);
}

fn price(plan: usize, period: usize) -> f64 {
    match (plan, period) {
        (0, 0) => 0.99,
        (0, 1) => 6.0,
        (1, 0) => 4.0,
        _ => 12.5,
    }
}

// ===== APP =====
impl LudoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        let mut rng = Rng::new();
        let (tex_static, a0) = load_tex(&cc.egui_ctx, "static", rgb(0x9b9ba6));
        let (tex_neon, a1) = load_tex(&cc.egui_ctx, "neon", rgb(0x7dd3a0));
        let title = TITLES[(rng.u64() % TITLES.len() as u64) as usize].to_string();
        let mut app = Self {
            rng,
            last_t: -1.0,
            title,
            title_since: 0.0,
            title_next: 24.0,
            maximized: false,
            theme_idx: 0,
            smooth_anim: true,
            custom_titlebar: true,
            deco_applied: false,
            auto_inject: false,
            servers: Vec::new(),
            server_input: String::new(),
            hook_decoy: false,
            new_visual: true,
            xltt_trim: "-2".into(),
            inject_cheat: true,
            deep_open: false,
            ring_upper: false,
            fake_client: false,
            fake_client_idx: 1,
            anti_spy: true,
            broken_kick: false,
            hide_bytecode: true,
            hide_bytecode_idx: 2,
            bg_particles: true,
            bg_particle_kind: 1,
            last_particle_kind: 1,
            particles: Vec::new(),
            particles_seeded: false,
            keys: Vec::new(),
            selected_key: 0,
            key_menu: None,
            nick_input: String::new(),
            pass_input: String::new(),
            account: None,
            unbind_input: String::new(),
            modal: None,
            buy_plan: 0,
            buy_period: 0,
            buy_until: 0.0,
            buy_dd_open: false,
            buy_error: String::new(),
            buy_result: None,
            copy_fb_until: 0.0,
            card_name: String::new(),
            card_number: String::new(),
            card_expiry: String::new(),
            card_cvv: String::new(),
            log: Vec::new(),
            toast: None,
            bars_t: 0.0,
            tex_static,
            tex_neon,
            aspects: [a0, a1],
            toys: Vec::new(),
            toys_seeded: false,
            next_spawn: 9.0,
            last_bounds: Rect::NOTHING,
            cfg: config_path(),
            last_sig: String::new(),
            dirty: false,
            last_save: 0.0,
        };
        app.load_cfg();
        app.last_particle_kind = app.bg_particle_kind;
        app.last_sig = app.sig();
        app.push_log("loader v2.1 ready");
        app.push_log("hwid scan ok");
        app
    }

    // ---- config ----
    fn sig(&self) -> String {
        let keys: Vec<String> = self
            .keys
            .iter()
            .map(|k| format!("{}|{}|{}", k.plan, k.key, k.bound as u8))
            .collect();
        let acc = self
            .account
            .as_ref()
            .map(|p| format!("{}|{}|{}", p.nick, p.hwid, p.plan))
            .unwrap_or_else(|| "-".into());
        format!(
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.theme_idx,
            self.smooth_anim as u8,
            self.custom_titlebar as u8,
            self.auto_inject as u8,
            self.servers.join(","),
            self.hook_decoy as u8,
            self.new_visual as u8,
            self.xltt_trim,
            self.inject_cheat as u8,
            self.selected_key,
            keys.join(";"),
            acc,
            self.deep_open as u8,
            self.ring_upper as u8,
            self.fake_client as u8,
            self.fake_client_idx,
            self.anti_spy as u8,
            self.broken_kick as u8,
            self.hide_bytecode as u8,
            self.hide_bytecode_idx,
            self.bg_particles as u8,
            self.bg_particle_kind,
        )
    }

    fn save_cfg(&self) {
        let mut s = String::new();
        s.push_str(&format!("theme={}\n", self.theme_idx));
        s.push_str(&format!("smooth={}\n", self.smooth_anim as u8));
        s.push_str(&format!("titlebar={}\n", self.custom_titlebar as u8));
        s.push_str(&format!("auto={}\n", self.auto_inject as u8));
        s.push_str(&format!("servers={}\n", self.servers.join(",")));
        s.push_str(&format!("hook={}\n", self.hook_decoy as u8));
        s.push_str(&format!("newvisual={}\n", self.new_visual as u8));
        s.push_str(&format!("xltt={}\n", self.xltt_trim));
        s.push_str(&format!("inject={}\n", self.inject_cheat as u8));
        s.push_str(&format!("selected={}\n", self.selected_key));
        s.push_str(&format!("deep={}\n", self.deep_open as u8));
        s.push_str(&format!("ring={}\n", self.ring_upper as u8));
        s.push_str(&format!("fake={}\n", self.fake_client as u8));
        s.push_str(&format!("fake_idx={}\n", self.fake_client_idx));
        s.push_str(&format!("spy={}\n", self.anti_spy as u8));
        s.push_str(&format!("kick={}\n", self.broken_kick as u8));
        s.push_str(&format!("hide={}\n", self.hide_bytecode as u8));
        s.push_str(&format!("hide_idx={}\n", self.hide_bytecode_idx));
        s.push_str(&format!("parts={}\n", self.bg_particles as u8));
        s.push_str(&format!("parts_kind={}\n", self.bg_particle_kind));
        for k in &self.keys {
            s.push_str(&format!("key={}||{}||{}\n", k.plan, k.key, k.bound as u8));
        }
        if let Some(p) = &self.account {
            s.push_str(&format!("nick={}\n", p.nick));
            s.push_str(&format!("hwid={}\n", p.hwid));
            s.push_str(&format!("plan={}\n", p.plan));
        }
        let _ = std::fs::write(&self.cfg, s);
    }

    fn load_cfg(&mut self) {
        let Ok(text) = std::fs::read_to_string(&self.cfg) else {
            return;
        };
        let mut nick = String::new();
        let mut hwid = String::new();
        let mut plan = String::new();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match k {
                "theme" => self.theme_idx = v.parse::<usize>().unwrap_or(0).min(5),
                "smooth" => self.smooth_anim = v == "1",
                "titlebar" => self.custom_titlebar = v == "1",
                "auto" => self.auto_inject = v == "1",
                "servers" => {
                    self.servers = v
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect()
                }
                "hook" => self.hook_decoy = v == "1",
                "newvisual" => self.new_visual = v == "1",
                "xltt" => self.xltt_trim = v.to_string(),
                "inject" => self.inject_cheat = v == "1",
                "selected" => self.selected_key = v.parse().unwrap_or(0),
                "deep" => self.deep_open = v == "1",
                "ring" => self.ring_upper = v == "1",
                "fake" => self.fake_client = v == "1",
                "fake_idx" => self.fake_client_idx = v.parse::<usize>().unwrap_or(1).min(4),
                "spy" => self.anti_spy = v == "1",
                "kick" => self.broken_kick = v == "1",
                "hide" => self.hide_bytecode = v == "1",
                "hide_idx" => self.hide_bytecode_idx = v.parse::<usize>().unwrap_or(2).min(3),
                "parts" => self.bg_particles = v == "1",
                "parts_kind" => self.bg_particle_kind = v.parse::<usize>().unwrap_or(1).min(3),
                "key" => {
                    let parts: Vec<&str> = v.splitn(3, "||").collect();
                    if parts.len() == 3 {
                        self.keys.push(KeyEntry {
                            plan: parts[0].to_string(),
                            key: parts[1].to_string(),
                            bound: parts[2] == "1",
                        });
                    }
                }
                "nick" => nick = v.to_string(),
                "hwid" => hwid = v.to_string(),
                "plan" => plan = v.to_string(),
                _ => {}
            }
        }
        if !nick.is_empty() {
            self.account = Some(Profile { nick, hwid, plan });
        }
        if self.selected_key >= self.keys.len() {
            self.selected_key = self.keys.len().saturating_sub(1);
        }
        self.sync_plan();
    }

    fn sync_plan(&mut self) {
        let plan = self.keys.get(self.selected_key).map(|k| k.plan.clone());
        if let (Some(p), Some(plan)) = (&mut self.account, plan) {
            p.plan = plan;
        }
    }

    fn push_log(&mut self, msg: &str) {
        self.log.push(LogLine {
            time: utc_hhmm(),
            msg: msg.to_string(),
        });
        if self.log.len() > 80 {
            let ex = self.log.len() - 80;
            self.log.drain(..ex);
        }
    }

    fn toast_msg(&mut self, msg: &str) {
        self.toast = Some((msg.to_string(), self.last_t + 2.5));
    }

    fn make_particle(&mut self, bounds: Rect, want: usize, anywhere: bool) -> Particle {
        let kind = pick_kind(want, &mut self.rng);
        let (size, vy) = match kind {
            0 => (self.rng.range(5.0, 9.0), self.rng.range(14.0, 28.0)),
            1 => (self.rng.range(4.0, 8.0), self.rng.range(9.0, 18.0)),
            _ => (self.rng.range(14.0, 22.0), self.rng.range(10.0, 20.0)),
        };
        let base_x = self.rng.range(bounds.left(), bounds.right());
        let y = if anywhere {
            self.rng.range(bounds.top(), bounds.bottom())
        } else {
            bounds.top() - size - self.rng.range(0.0, 60.0)
        };
        Particle {
            kind,
            base_x,
            pos: pos2(base_x, y),
            vy,
            sway_amp: self.rng.range(6.0, 20.0),
            sway_freq: self.rng.range(0.25, 0.9),
            phase: self.rng.range(0.0, 6.283),
            size,
            angle: self.rng.range(0.0, 6.283),
            ang_vel: self.rng.range(-0.6, 0.6),
            alpha: self.rng.range(28.0, 80.0) as u8,
        }
    }

    fn particles_frame(&mut self, p: &egui::Painter, bounds: Rect, dt: f32, now: f64) {
        if !self.bg_particles {
            return;
        }
        if !self.particles_seeded && bounds.width() > 100.0 {
            for _ in 0..48 {
                self.particles
                    .push(self.make_particle(bounds, self.bg_particle_kind, true));
            }
            self.particles_seeded = true;
        }
        if self.last_particle_kind != self.bg_particle_kind {
            self.last_particle_kind = self.bg_particle_kind;
            let want = self.bg_particle_kind;
            for pa in &mut self.particles {
                pa.kind = pick_kind(want, &mut self.rng);
            }
        }

        for pa in &mut self.particles {
            pa.pos.y += pa.vy * dt;
            pa.pos.x = pa.base_x + (now as f32 * pa.sway_freq + pa.phase).sin() * pa.sway_amp;
            pa.angle += pa.ang_vel * dt;
        }

        let mut respawn: Vec<usize> = Vec::new();
        for (i, pa) in self.particles.iter().enumerate() {
            if pa.pos.y - pa.size > bounds.bottom() {
                respawn.push(i);
            }
        }
        for i in respawn {
            self.particles[i] = self.make_particle(bounds, self.bg_particle_kind, false);
        }

        let tex = self.tex_static.clone();
        let aspect = self.aspects[0];
        for pa in &self.particles {
            let col = Color32::from_white_alpha(pa.alpha);
            match pa.kind {
                0 => draw_snow(p, pa.pos, pa.size, pa.angle, col),
                1 => draw_star(p, pa.pos, pa.size, pa.angle, col),
                _ => draw_particle_icon(p, &tex, pa.pos, pa.size, pa.angle, col, aspect),
            }
        }
    }

    fn spawn_toy(&mut self, bounds: Rect, count: usize) {
        if bounds.width() < 100.0 {
            return;
        }
        for _ in 0..count {
            let size = self.rng.range(44.0, 62.0);
            let x = self.rng.range(bounds.left() + size, bounds.right() - size);
            self.toys.push(Toy {
                tex: (self.rng.u64() % 2) as usize,
                pos: pos2(x, bounds.top() - size),
                vel: vec2(self.rng.range(-25.0, 25.0), 30.0),
                angle: self.rng.range(-0.5, 0.5),
                ang_vel: self.rng.range(-0.8, 0.8),
                size,
                squash: 0.0,
                grabbed: false,
                was_grabbed: false,
            });
        }
    }

    fn open_buy(&mut self) {
        self.buy_result = None;
        self.buy_error.clear();
        self.buy_dd_open = false;
        self.card_name.clear();
        self.card_number.clear();
        self.card_expiry.clear();
        self.card_cvv.clear();
        self.modal = Some(Modal::Buy);
    }

    fn card_empty(&self) -> bool {
        self.card_name.trim().is_empty()
            || self.card_number.trim().is_empty()
            || self.card_expiry.trim().is_empty()
            || self.card_cvv.trim().is_empty()
    }

    fn card_valid(&self) -> bool {
        let name_ok = self.card_name.trim().chars().count() >= 3;
        let digits: String = self
            .card_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let num_ok = digits.len() >= 15 && digits.len() <= 16;
        let e = self.card_expiry.as_bytes();
        let exp_ok = e.len() == 5
            && e[2] == b'/'
            && e[0].is_ascii_digit()
            && e[1].is_ascii_digit()
            && e[3].is_ascii_digit()
            && e[4].is_ascii_digit();
        let cvv_ok = (3..=4).contains(&self.card_cvv.len())
            && self.card_cvv.chars().all(|c| c.is_ascii_digit());
        name_ok && num_ok && exp_ok && cvv_ok
    }

    // ===== SIDEBAR: КЛИЕНТ =====
    fn client_section(&mut self, ui: &mut egui::Ui, t: &Theme) {
        section_head(ui, t, "Клиент", 0.0);
        toggle_row(ui, t, "Выборочная инъекция", &mut self.auto_inject, self.smooth_anim);
        if self.auto_inject {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(t.s2)
                .rounding(8.0)
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Серверы для выборочной инъекции").size(12.0).color(t.t2));
                    ui.add_space(8.0);
                    let mut remove: Option<usize> = None;
                    if self.servers.is_empty() {
                        ui.label(
                            RichText::new("Список пуст — инъекция под все серверы")
                                .size(11.0)
                                .color(t.t3),
                        );
                    } else {
                        egui::Grid::new("srv_table")
                            .num_columns(2)
                            .min_col_width(0.0)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                for (i, s) in self.servers.iter().enumerate() {
                                    ui.label(RichText::new(s).monospace().size(12.0).color(t.text));
                                    let (r, resp) =
                                        ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    let c = r.center();
                                    let col = if resp.hovered() { t.text } else { t.t3 };
                                    let st = Stroke::new(1.0, col);
                                    ui.painter().line_segment(
                                        [pos2(c.x - 4.0, c.y - 4.0), pos2(c.x + 4.0, c.y + 4.0)],
                                        st,
                                    );
                                    ui.painter().line_segment(
                                        [pos2(c.x - 4.0, c.y + 4.0), pos2(c.x + 4.0, c.y - 4.0)],
                                        st,
                                    );
                                    if resp.clicked() {
                                        remove = Some(i);
                                    }
                                    ui.end_row();
                                }
                            });
                    }
                    if let Some(i) = remove {
                        self.servers.remove(i);
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let w = ui.available_width() - 40.0;
                        let input_id = egui::Id::new("selective_server_input");
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.server_input)
                                .id(input_id)
                                .hint_text("pvp.su")
                                .desired_width(w)
                                .min_size(vec2(0.0, 32.0))
                                .font(egui::TextStyle::Monospace),
                        );
                        let enter = r.lost_focus()
                            && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter));
                        let plus = ui
                            .add(
                                egui::Button::new(RichText::new("+").size(14.0).color(t.text))
                                    .fill(t.s3)
                                    .stroke(Stroke::new(1.0, t.line2))
                                    .rounding(8.0)
                                    .min_size(vec2(32.0, 32.0)),
                            )
                            .clicked();
                        if enter || plus {
                            let s = self.server_input.trim().to_string();
                            if !s.is_empty() {
                                if self.servers.contains(&s) {
                                    self.toast_msg("Сервер уже в списке");
                                } else {
                                    self.servers.push(s);
                                }
                                self.server_input.clear();
                            }
                            ui.ctx().memory_mut(|m| m.request_focus(input_id));
                        }
                    });
                });
            ui.add_space(4.0);
        }
        ui.add_space(8.0);
        if ghost_button(ui, t, "Библиотека ключей").clicked() {
            self.modal = Some(Modal::Keys);
        }
        if let Some(k) = self.keys.get(self.selected_key) {
            ui.label(
                RichText::new(format!("{} · {}…", k.plan, &k.key[..14.min(k.key.len())]))
                    .monospace()
                    .size(11.0)
                    .color(t.t3),
            );
        } else {
            ui.label(RichText::new("Ключ не выбран").monospace().size(11.0).color(t.t3));
        }
        ui.add_space(8.0);
        toggle_row(ui, t, "Хук-обманка анти-читов", &mut self.hook_decoy, self.smooth_anim);
        toggle_row(ui, t, "Новый визуал чит-клиента", &mut self.new_visual, self.smooth_anim);
        ui.add_space(8.0);

        // ===== Глубокая настройка =====
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
        ui.painter().text(
            pos2(rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            "Глубокая настройка",
            FontId::proportional(13.0),
            t.t2,
        );
        let ch = pos2(rect.right() - 12.0, rect.center().y);
        let st = Stroke::new(1.5, t.t3);
        if self.deep_open {
            ui.painter()
                .line_segment([pos2(ch.x - 4.0, ch.y - 2.0), pos2(ch.x, ch.y + 2.0)], st);
            ui.painter()
                .line_segment([pos2(ch.x, ch.y + 2.0), pos2(ch.x + 4.0, ch.y - 2.0)], st);
        } else {
            ui.painter()
                .line_segment([pos2(ch.x - 2.0, ch.y - 4.0), pos2(ch.x + 2.0, ch.y)], st);
            ui.painter()
                .line_segment([pos2(ch.x + 2.0, ch.y), pos2(ch.x - 2.0, ch.y + 4.0)], st);
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.deep_open = !self.deep_open;
        }
        if self.deep_open {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(t.s2)
                .rounding(8.0)
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("XLTT-обрез пакета").size(13.0).color(t.t2));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.xltt_trim)
                                    .desired_width(56.0)
                                    .horizontal_align(Align::RIGHT)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                    });
                    ui.add_space(8.0);
                    toggle_row(ui, t, "Ring-Upper обход", &mut self.ring_upper, self.smooth_anim);
                    ui.add_space(8.0);
                    toggle_row(ui, t, "Fake client", &mut self.fake_client, self.smooth_anim);
                    ui.add_space(4.0);
                    field_label(ui, t, "Поддельный клиент");
                    let w = ui.available_width();
                    egui::ComboBox::from_id_salt("fake_client_select")
                        .width(w)
                        .selected_text(FAKE_CLIENTS[self.fake_client_idx])
                        .show_ui(ui, |ui| {
                            for (i, opt) in FAKE_CLIENTS.iter().enumerate() {
                                ui.selectable_value(&mut self.fake_client_idx, i, *opt);
                            }
                        });
                    ui.add_space(8.0);
                    toggle_row(ui, t, "Anti client screen spy", &mut self.anti_spy, self.smooth_anim);
                    ui.add_space(8.0);
                    toggle_row(ui, t, "Broken kick packets", &mut self.broken_kick, self.smooth_anim);
                    ui.add_space(8.0);
                    toggle_row(ui, t, "Hide bytecode", &mut self.hide_bytecode, self.smooth_anim);
                    ui.add_space(4.0);
                    field_label(ui, t, "Способ скрытия");
                    let w = ui.available_width();
                    egui::ComboBox::from_id_salt("hide_bytecode_select")
                        .width(w)
                        .selected_text(HIDE_BYTECODE[self.hide_bytecode_idx])
                        .show_ui(ui, |ui| {
                            for (i, opt) in HIDE_BYTECODE.iter().enumerate() {
                                ui.selectable_value(&mut self.hide_bytecode_idx, i, *opt);
                            }
                        });
                    ui.add_space(4.0);
                });
        }
    }

    // ===== SIDEBAR: КАСТОМИЗАЦИЯ =====
    fn custom_section(&mut self, ui: &mut egui::Ui, t: &Theme, ctx: &egui::Context) {
        section_head(ui, t, "Кастомизация", 24.0);
        ui.label(RichText::new("Тема приложения").size(13.0).color(t.t2));
        ui.add_space(8.0);
        let w = (ui.available_width() - 16.0) / 3.0;
        for row in 0..2 {
            ui.horizontal(|ui| {
                for col in 0..3 {
                    let i = row * 3 + col;
                    let th = THEMES[i];
                    let (rect, resp) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
                    if resp.clicked() {
                        self.theme_idx = i;
                    }
                    let sel = i == self.theme_idx;
                    ui.painter()
                        .rect_filled(rect, 8.0, if sel { th.s3 } else { th.s2 });
                    ui.painter().rect_stroke(
                        rect,
                        8.0,
                        Stroke::new(1.0, if sel { th.accent } else { th.line2 }),
                    );
                    ui.painter()
                        .circle_filled(pos2(rect.left() + 13.0, rect.center().y), 4.0, th.accent);
                    ui.painter().text(
                        pos2(rect.left() + 24.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        th.name,
                        FontId::proportional(11.5),
                        if sel { th.text } else { th.t2 },
                    );
                    if resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
        }
        ui.add_space(8.0);
        toggle_row(ui, t, "Задний план", &mut self.bg_particles, self.smooth_anim);
        if self.bg_particles {
            ui.add_space(4.0);
            field_label(ui, t, "Тип частиц");
            let w = ui.available_width();
            egui::ComboBox::from_id_salt("bg_particle_kind")
                .width(w)
                .selected_text(PARTICLE_KINDS[self.bg_particle_kind])
                .show_ui(ui, |ui| {
                    for (i, opt) in PARTICLE_KINDS.iter().enumerate() {
                        ui.selectable_value(&mut self.bg_particle_kind, i, *opt);
                    }
                });
        }
        ui.add_space(8.0);
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
        section_head(ui, t, "Прочее", 24.0);
        if ghost_button(ui, t, "Сайт проекта").clicked() {
            let _ = open::that("https://ludoclient.github.io");
            self.toast_msg("Открываю ludoclient.github.io");
        }
        ui.label(
            RichText::new("ludoclient.github.io")
                .monospace()
                .size(10.0)
                .color(t.t3),
        );
        ui.add_space(8.0);
        let w = ui.available_width();
        let btn = egui::Button::new(RichText::new("Отвязать все ключи").size(13.0).color(t.danger))
            .fill(t.s2)
            .stroke(Stroke::new(1.0, mix(t.danger, t.line2, 0.55)))
            .rounding(8.0)
            .min_size(vec2(w, 36.0));
        if ui.add(btn).clicked() {
            self.unbind_input.clear();
            self.modal = Some(Modal::Unbind);
        }
    }

    // ===== SIDEBAR: АККАУНТ =====
    fn account_section(&mut self, ui: &mut egui::Ui, t: &Theme, ctx: &egui::Context) {
        section_head(ui, t, "Аккаунт", 24.0);
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 56.0), Sense::click());
        let hover = resp.hovered();
        ui.painter().rect_filled(rect, 12.0, t.s2);
        ui.painter().rect_stroke(
            rect,
            12.0,
            Stroke::new(1.0, if hover { t.t3 } else { t.line }),
        );
        if hover {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let ac = pos2(rect.left() + 28.0, rect.center().y);
        let p = ui.painter();
        if let Some(acc) = &self.account {
            p.circle_filled(ac, 16.0, t.accent_dim);
            let initial = acc
                .nick
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_else(|| "?".into());
            p.text(ac, Align2::CENTER_CENTER, initial, FontId::proportional(14.0), t.bg);
            p.text(
                pos2(rect.left() + 52.0, rect.center().y - 8.0),
                Align2::LEFT_CENTER,
                &acc.nick,
                FontId::proportional(14.0),
                t.text,
            );
            p.text(
                pos2(rect.left() + 52.0, rect.center().y + 10.0),
                Align2::LEFT_CENTER,
                format!("{} · HWID {}", acc.plan, acc.hwid),
                FontId::monospace(10.0),
                t.t3,
            );
        } else {
            p.circle_stroke(ac, 16.0, Stroke::new(1.0, t.t3));
            let clip = Rect::from_center_size(ac, vec2(30.0, 30.0));
            let cp = p.with_clip_rect(clip);
            cp.circle_filled(pos2(ac.x, ac.y - 4.0), 5.0, t.t3);
            cp.circle_filled(pos2(ac.x, ac.y + 12.0), 9.0, t.t3);
            p.text(
                pos2(rect.left() + 52.0, rect.center().y - 8.0),
                Align2::LEFT_CENTER,
                "Нет аккаунта",
                FontId::proportional(14.0),
                t.text,
            );
            p.text(
                pos2(rect.left() + 52.0, rect.center().y + 10.0),
                Align2::LEFT_CENTER,
                "Нажмите, чтобы войти",
                FontId::monospace(10.0),
                t.t3,
            );
        }
        let ch = pos2(rect.right() - 16.0, rect.center().y);
        let st = Stroke::new(1.0, t.t3);
        p.line_segment([pos2(ch.x - 2.0, ch.y - 4.0), pos2(ch.x + 2.0, ch.y)], st);
        p.line_segment([pos2(ch.x + 2.0, ch.y), pos2(ch.x - 2.0, ch.y + 4.0)], st);
        if resp.clicked() {
            self.modal = Some(Modal::Auth);
        }
    }

    // ===== SIDEBAR: BAN-STAT =====
    fn banstat_panel(&mut self, ctx: &egui::Context, t: &Theme, dt: f32) {
        if !self.smooth_anim {
            self.bars_t = 1.0;
        } else {
            self.bars_t = (self.bars_t + dt * 1.4).min(1.0);
        }
        let ease = 1.0 - (1.0 - self.bars_t).powi(3);

        egui::SidePanel::right("banstat")
            .exact_width(252.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(t.bg).inner_margin(Margin::symmetric(16.0, 20.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("bs")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Статистика банов").size(17.0).strong().color(t.text));
                        ui.label(
                            RichText::new("по античитам · последние 30 дней")
                                .monospace()
                                .size(10.0)
                                .color(t.t3),
                        );
                        ui.add_space(12.0);

                        const DATA: [(&str, f32, f32); 4] = [
                            ("Valcuria", 12.0, 2.4),
                            ("PaperPlate", 7.0, -1.2),
                            ("NoHack", 4.0, -0.6),
                            ("NeoAC", 1.0, -0.3),
                        ];
                        let max = DATA.iter().map(|d| d.1).fold(0.0f32, |a, b| a.max(b));

                        let cw = (ui.available_width() - 8.0) / 2.0;
                        for pair in DATA.chunks(2) {
                            ui.horizontal(|ui| {
                                for (name, pct, chg) in pair {
                                    ui.allocate_ui_with_layout(
                                        vec2(cw, 88.0),
                                        Layout::top_down(Align::LEFT),
                                        |ui| {
                                            egui::Frame::none()
                                                .fill(t.s1)
                                                .stroke(Stroke::new(1.0, t.line))
                                                .rounding(12.0)
                                                .inner_margin(Margin::same(12.0))
                                                .show(ui, |ui| {
                                                    ui.set_min_width(cw - 24.0);
                                                    ui.spacing_mut().item_spacing.y = 4.0;
                                                    ui.label(
                                                        RichText::new(*name)
                                                            .monospace()
                                                            .size(10.0)
                                                            .color(t.t2),
                                                    );
                                                    let lead = *pct >= max;
                                                    ui.label(
                                                        RichText::new(format!("{}%", pct))
                                                            .size(22.0)
                                                            .strong()
                                                            .color(if lead { t.accent } else { t.text }),
                                                    );
                                                    let up = *chg > 0.0;
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{}{:.1}%",
                                                            if up { "▲ +" } else { "▼ " },
                                                            chg
                                                        ))
                                                        .monospace()
                                                        .size(10.0)
                                                        .color(if up { t.danger } else { t.accent }),
                                                    );
                                                });
                                        },
                                    );
                                }
                            });
                            ui.add_space(4.0);
                        }

                        ui.add_space(12.0);
                        ui.label(RichText::new("Баны по античитам").size(13.0).strong().color(t.text));
                        ui.label(
                            RichText::new("процент забаненных за 30 дней")
                                .monospace()
                                .size(10.0)
                                .color(t.t3),
                        );
                        ui.add_space(8.0);
                        for (name, pct, _) in DATA {
                            let lead = pct >= max;
                            let (row, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 16.0),
                                Sense::hover(),
                            );
                            let p = ui.painter();
                            p.text(
                                pos2(row.left(), row.center().y),
                                Align2::LEFT_CENTER,
                                name,
                                FontId::monospace(10.5),
                                t.t2,
                            );
                            let track = Rect::from_min_max(
                                pos2(row.left() + 76.0, row.center().y - 4.0),
                                pos2(row.right() - 36.0, row.center().y + 4.0),
                            );
                            p.rect_filled(track, 4.0, t.s3);
                            let w = (track.width() * (pct / max) * ease).max(8.0);
                            let fill = Rect::from_min_size(track.min, vec2(w, 8.0));
                            p.rect_filled(fill, 4.0, if lead { t.accent } else { t.t3 });
                            p.text(
                                pos2(row.right(), row.center().y),
                                Align2::RIGHT_CENTER,
                                format!("{}%", pct),
                                FontId::monospace(10.5),
                                t.t2,
                            );
                            ui.add_space(8.0);
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("обновлено: {} · 04:12 UTC", utc_date()))
                                .monospace()
                                .size(10.0)
                                .color(t.t3),
                        );
                        ui.label(
                            RichText::new("источник: телеметрия лоадера")
                                .monospace()
                                .size(10.0)
                                .color(t.t3),
                        );
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
            let size = self.toys[i].size + 12.0;
            let r = size * 0.5;
            let hit = Rect::from_center_size(self.toys[i].pos, vec2(size, size));
            let resp = ui.interact(hit, egui::Id::new("toy").with(i), Sense::drag());

            let toy = &mut self.toys[i];
            toy.was_grabbed = toy.grabbed;
            toy.grabbed = resp.dragged();

            if toy.grabbed {
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
                    if sp > 1500.0 {
                        toy.vel *= 1500.0 / sp;
                    }
                }
                toy.vel.y += 230.0 * dt;
                toy.vel *= (1.0 - 0.5 * dt).max(0.0);
                if toy.vel.y > 120.0 {
                    toy.vel.y = 120.0;
                }
                toy.pos += toy.vel * dt;
                toy.angle += toy.ang_vel * dt;
                toy.ang_vel *= (1.0 - 1.2 * dt).max(0.0);

                if toy.pos.x - r < bounds.left() {
                    toy.pos.x = bounds.left() + r;
                    if toy.vel.x < 0.0 {
                        toy.vel.x = -toy.vel.x * 0.5;
                        toy.ang_vel -= toy.vel.y * 0.002;
                    }
                }
                if toy.pos.x + r > bounds.right() {
                    toy.pos.x = bounds.right() - r;
                    if toy.vel.x > 0.0 {
                        toy.vel.x = -toy.vel.x * 0.5;
                        toy.ang_vel += toy.vel.y * 0.002;
                    }
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
                        if toy.vel.y.abs() < 28.0 {
                            toy.vel.y = 0.0;
                        }
                    }
                    toy.vel.x *= (1.0 - 3.0 * dt).max(0.0);
                    toy.ang_vel -= toy.angle.sin() * 1.5 * dt;
                }
                for o in obstacles {
                    bounce_rect(toy, *o);
                }
            }
            toy.squash = (toy.squash - 3.5 * dt).max(0.0);
            if resp.hovered() && cursor.is_none() {
                cursor = Some(egui::CursorIcon::Grab);
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let min = (self.toys[i].size + self.toys[j].size) * 0.5 + 12.0;
                let d = self.toys[j].pos - self.toys[i].pos;
                let dist = d.length();
                if dist < min && dist > 0.001 {
                    let nrm = d / dist;
                    let pen = min - dist;
                    let (gi, gj) = (self.toys[i].grabbed, self.toys[j].grabbed);
                    let (wi, wj) = match (gi, gj) {
                        (true, true) => (0.0, 0.0),
                        (true, false) => (0.0, 1.0),
                        (false, true) => (1.0, 0.0),
                        _ => (0.5, 0.5),
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
                        if rv < -80.0 {
                            self.toys[i].squash = 0.5;
                            self.toys[j].squash = 0.5;
                        }
                    }
                }
            }
        }

        let painter = ui.painter().clone();
        for toy in &self.toys {
            let tex = if toy.tex == 0 { &self.tex_static } else { &self.tex_neon };
            draw_toy(&painter, tex, toy, self.aspects[toy.tex]);
        }
        if let Some(c) = cursor {
            ui.ctx().set_cursor_icon(c);
        }
    }

    // ===== MODAL: БИБЛИОТЕКА КЛЮЧЕЙ =====
    fn keys_modal(&mut self, ui: &mut egui::Ui, t: &Theme, ctx: &egui::Context) {
        ui.label(RichText::new("Библиотека ключей").size(18.0).strong().color(t.text));
        ui.label(RichText::new("Выберите активный ключ").size(12.0).color(t.t2));
        ui.add_space(12.0);
        if self.keys.is_empty() {
            egui::Frame::none()
                .fill(t.s2)
                .rounding(8.0)
                .inner_margin(Margin::same(24.0))
                .show(ui, |ui| {
                    ui.set_min_width(372.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("У вас нету ключей :(").size(14.0).color(t.t3));
                    });
                });
        } else {
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..self.keys.len() {
                        let sel = i == self.selected_key;
                        let k = &self.keys[i];
                        let fr = egui::Frame::none()
                            .fill(if sel { t.s3 } else { t.s2 })
                            .rounding(8.0)
                            .stroke(Stroke::new(1.0, if sel { t.accent_dim } else { t.line }))
                            .inner_margin(Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_min_width(368.0);
                                ui.horizontal(|ui| {
                                    let (dot, _) =
                                        ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                                    if sel {
                                        ui.painter().circle_filled(dot.center(), 5.0, t.accent);
                                    } else {
                                        ui.painter()
                                            .circle_stroke(dot.center(), 5.0, Stroke::new(1.0, t.t3));
                                    }
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&k.plan)
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(t.accent),
                                            );
                                            ui.label(
                                                RichText::new(if k.bound { "привязан" } else { "свободен" })
                                                    .monospace()
                                                    .size(10.0)
                                                    .color(if k.bound { t.t3 } else { t.accent_dim }),
                                            );
                                        });
                                        ui.label(
                                            RichText::new(short_key(&k.key))
                                                .monospace()
                                                .size(12.0)
                                                .color(t.text),
                                        );
                                    });
                                });
                            });
                        let resp = ui.interact(
                            fr.response.rect,
                            egui::Id::new("keyrow").with(i),
                            Sense::click(),
                        );
                        if resp.clicked() {
                            self.selected_key = i;
                            self.sync_plan();
                        }
                        if resp.secondary_clicked() {
                            if let Some(p) = ctx.input(|i| i.pointer.hover_pos()) {
                                self.key_menu = Some((i, p));
                            }
                        }
                        if resp.hovered() {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            resp.clone().on_hover_text("ПКМ — меню ключа");
                        }
                        ui.add_space(8.0);
                    }
                });
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Купить ключ").size(13.0).color(t.bg))
                        .fill(t.text)
                        .stroke(Stroke::NONE)
                        .rounding(8.0)
                        .min_size(vec2(180.0, 36.0)),
                )
                .clicked()
            {
                self.open_buy();
            }
            let has = !self.keys.is_empty();
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(RichText::new("Копировать").size(13.0).color(t.text))
                        .fill(t.s3)
                        .stroke(Stroke::new(1.0, t.line2))
                        .rounding(8.0)
                        .min_size(vec2(120.0, 36.0)),
                )
                .clicked()
            {
                if let Some(k) = self.keys.get(self.selected_key) {
                    ctx.copy_text(k.key.clone());
                    self.toast_msg("Ключ скопирован");
                }
            }
        });
        ui.add_space(12.0);
        ui.label(
            RichText::new("Ключ активируется при первом входе в лаунчер и привязывается к железу")
                .size(11.0)
                .color(t.t3),
        );
    }

    // ===== MODAL: ОПЛАТА (как на сайте) =====
    fn buy_modal(&mut self, ui: &mut egui::Ui, t: &Theme, ctx: &egui::Context, now: f64) {
        if let Some(key) = self.buy_result.clone() {
            ui.label(RichText::new("Платёж обработан").size(18.0).strong().color(t.text));
            ui.label(RichText::new("Ваш ключ LudoClient:").size(13.0).color(t.t2));
            ui.add_space(16.0);
            egui::Frame::none()
                .fill(t.s2)
                .stroke(Stroke::new(1.0, t.line2))
                .rounding(8.0)
                .inner_margin(Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_min_width(388.0);
                    ui.label(RichText::new("Лицензионный ключ").size(12.0).color(t.t3));
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(t.bg)
                        .stroke(Stroke::new(1.0, t.line))
                        .rounding(6.0)
                        .inner_margin(Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&key).monospace().size(12.0).color(t.accent));
                        });
                    ui.add_space(8.0);
                    let copied = now < self.copy_fb_until;
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(if copied { "✓ Скопировано!" } else { "Копировать ключ" })
                                    .size(12.0)
                                    .color(t.text),
                            )
                            .fill(t.s3)
                            .stroke(Stroke::new(1.0, t.line2))
                            .rounding(8.0)
                            .min_size(vec2(140.0, 32.0)),
                        )
                        .clicked()
                    {
                        ctx.copy_text(key.clone());
                        self.copy_fb_until = now + 2.0;
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Ключ активируется при первом входе в лаунчер и привязывается к железу")
                            .size(11.0)
                            .color(t.t3),
                    );
                });
            ui.add_space(16.0);
            let w = ui.available_width();
            if ui
                .add(
                    egui::Button::new(RichText::new("В библиотеку").size(13.0).color(t.bg))
                        .fill(t.text)
                        .stroke(Stroke::NONE)
                        .rounding(8.0)
                        .min_size(vec2(w, 40.0)),
                )
                .clicked()
            {
                self.buy_result = None;
                self.modal = Some(Modal::Keys);
            }
            return;
        }

        let plan_name = if self.buy_plan == 0 { "Static" } else { "Prime" };
        let period_name = if self.buy_period == 0 { "месяц" } else { "год" };
        ui.label(
            RichText::new(format!("Подписка {} на {}", plan_name, period_name))
                .size(18.0)
                .strong()
                .color(t.text),
        );
        ui.label(
            RichText::new("Заполните данные карты. После оплаты ключ выдаётся автоматически")
                .size(13.0)
                .color(t.t2),
        );
        ui.add_space(16.0);

        for i in 0..2 {
            let sel = i == self.buy_plan;
            let fr = egui::Frame::none()
                .fill(if sel { t.s3 } else { t.s2 })
                .rounding(8.0)
                .stroke(Stroke::new(1.0, if sel { t.accent_dim } else { t.line }))
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.set_min_width(388.0);
                    ui.horizontal(|ui| {
                        let (dot, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        if sel {
                            ui.painter().circle_filled(dot.center(), 5.0, t.accent);
                        } else {
                            ui.painter()
                                .circle_stroke(dot.center(), 5.0, Stroke::new(1.0, t.t3));
                        }
                        ui.label(
                            RichText::new(if i == 0 { "Static" } else { "Prime" })
                                .size(14.0)
                                .color(t.text),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("${:.2}/мес · ${:.2}/год", price(i, 0), price(i, 1)))
                                    .monospace()
                                    .size(11.0)
                                    .color(t.t3),
                            );
                        });
                    });
                });
            let resp = ui.interact(fr.response.rect, egui::Id::new("buyplan").with(i), Sense::click());
            if resp.clicked() {
                self.buy_plan = i;
                self.buy_error.clear();
                self.buy_dd_open = false;
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            ui.add_space(8.0);
        }

        ui.add_space(4.0);
        let w = ui.available_width();
        let (trig, trig_resp) = ui.allocate_exact_size(vec2(w, 44.0), Sense::click());
        let open = self.buy_dd_open;
        ui.painter().rect_filled(trig, 8.0, t.bg);
        ui.painter().rect_stroke(
            trig,
            8.0,
            Stroke::new(1.0, if open { t.accent } else if trig_resp.hovered() { t.t3 } else { t.line2 }),
        );
        let label = format!(
            "{} подписка — ${:.2}",
            if self.buy_period == 0 { "Месячная" } else { "Годовая" },
            price(self.buy_plan, self.buy_period)
        );
        ui.painter().text(
            pos2(trig.left() + 12.0, trig.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            t.text,
        );
        let ch = pos2(trig.right() - 16.0, trig.center().y);
        let st = Stroke::new(1.5, t.t2);
        if open {
            ui.painter().line_segment([pos2(ch.x - 4.0, ch.y + 2.0), pos2(ch.x, ch.y - 2.0)], st);
            ui.painter().line_segment([pos2(ch.x, ch.y - 2.0), pos2(ch.x + 4.0, ch.y + 2.0)], st);
        } else {
            ui.painter().line_segment([pos2(ch.x - 4.0, ch.y - 2.0), pos2(ch.x, ch.y + 2.0)], st);
            ui.painter().line_segment([pos2(ch.x, ch.y + 2.0), pos2(ch.x + 4.0, ch.y - 2.0)], st);
        }
        if trig_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if trig_resp.clicked() {
            self.buy_dd_open = !self.buy_dd_open;
        }
        if self.buy_dd_open {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(t.s2)
                .stroke(Stroke::new(1.0, t.line2))
                .rounding(8.0)
                .inner_margin(Margin::same(4.0))
                .show(ui, |ui| {
                    ui.set_min_width(w - 8.0);
                    for i in 0..2 {
                        let sel = i == self.buy_period;
                        let txt = format!(
                            "{} подписка — ${:.2}",
                            if i == 0 { "Месячная" } else { "Годовая" },
                            price(self.buy_plan, i)
                        );
                        let (r, resp) = ui.allocate_exact_size(vec2(w - 8.0, 32.0), Sense::click());
                        let fill = if sel || resp.hovered() { t.s3 } else { Color32::TRANSPARENT };
                        ui.painter().rect_filled(r, 6.0, fill);
                        ui.painter().text(
                            pos2(r.left() + 10.0, r.center().y),
                            Align2::LEFT_CENTER,
                            txt,
                            FontId::proportional(14.0),
                            if sel || resp.hovered() { t.text } else { t.t2 },
                        );
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            self.buy_period = i;
                            self.buy_dd_open = false;
                            self.buy_error.clear();
                        }
                    }
                });
        }

        ui.add_space(16.0);
        field_label(ui, t, "Имя держателя карты");
        let w = ui.available_width();
        ui.add(
            egui::TextEdit::singleline(&mut self.card_name)
                .hint_text("IVAN IVANOV")
                .desired_width(w)
                .min_size(vec2(0.0, 44.0))
                .font(egui::TextStyle::Monospace),
        );
        ui.add_space(12.0);
        field_label(ui, t, "Номер карты");
        let w = ui.available_width();
        let r = ui.add(
            egui::TextEdit::singleline(&mut self.card_number)
                .hint_text("0000 0000 0000 0000")
                .desired_width(w)
                .min_size(vec2(0.0, 44.0))
                .font(egui::TextStyle::Monospace),
        );
        if r.changed() {
            let digits: String = self
                .card_number
                .chars()
                .filter(|c| c.is_ascii_digit())
                .take(16)
                .collect();
            let mut f = String::new();
            for (i, ch) in digits.chars().enumerate() {
                if i > 0 && i % 4 == 0 {
                    f.push(' ');
                }
                f.push(ch);
            }
            self.card_number = f;
        }
        ui.add_space(12.0);
        let w = ui.available_width();
        let half = (w - 12.0) / 2.0;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                field_label(ui, t, "Срок действия");
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.card_expiry)
                        .hint_text("MM/YY")
                        .desired_width(half)
                        .min_size(vec2(0.0, 44.0))
                        .font(egui::TextStyle::Monospace),
                );
                if r.changed() {
                    let d: String = self
                        .card_expiry
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(4)
                        .collect();
                    self.card_expiry = if d.len() > 2 {
                        format!("{}/{}", &d[..2], &d[2..])
                    } else {
                        d
                    };
                }
            });
            ui.add_space(4.0);
            ui.vertical(|ui| {
                field_label(ui, t, "CVV / CVC");
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.card_cvv)
                        .hint_text("•••")
                        .password(true)
                        .desired_width(half)
                        .min_size(vec2(0.0, 44.0))
                        .font(egui::TextStyle::Monospace),
                );
                if r.changed() {
                    self.card_cvv = self
                        .card_cvv
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(4)
                        .collect();
                }
            });
        });
        ui.add_space(16.0);
        ui.label(
            RichText::new(format!("Сумма к оплате: ${:.2}", price(self.buy_plan, self.buy_period)))
                .size(14.0)
                .color(t.t2),
        );
        if !self.buy_error.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new(&self.buy_error).size(12.0).color(t.danger));
        }
        ui.add_space(16.0);
        let busy = self.buy_until > 0.0;
        let label = if busy { "Обработка..." } else { "Оплатить и получить ключ" };
        let w = ui.available_width();
        if ui
            .add_enabled(
                !busy,
                egui::Button::new(RichText::new(label).size(14.0).color(t.bg))
                    .fill(t.text)
                    .stroke(Stroke::NONE)
                    .rounding(8.0)
                    .min_size(vec2(w, 44.0)),
            )
            .clicked()
        {
            if self.card_empty() {
                self.buy_error = "Пожалуйста, заполните все поля платежной информации.".into();
            } else if !self.card_valid() {
                self.buy_error = "Проверьте правильность введенных данных карты.".into();
            } else {
                self.buy_error.clear();
                self.buy_until = now + 1.5;
            }
        }
    }

    fn auth_modal(&mut self, ui: &mut egui::Ui, t: &Theme) {
        if let Some(p) = self.account.clone() {
            ui.label(RichText::new(&p.nick).size(18.0).strong().color(t.text));
            ui.label(
                RichText::new(format!("{} · HWID {}", p.plan, p.hwid))
                    .monospace()
                    .size(11.0)
                    .color(t.t3),
            );
            ui.add_space(16.0);
            if ghost_button(ui, t, "Выйти").clicked() {
                self.push_log("session closed");
                self.account = None;
                self.modal = None;
            }
        } else {
            ui.label(RichText::new("Вход в аккаунт").size(18.0).strong().color(t.text));
            ui.label(RichText::new("Ник и пароль от профиля LudoClient").size(12.0).color(t.t2));
            ui.add_space(16.0);
            let w = ui.available_width();
            let r = ui.add(
                egui::TextEdit::singleline(&mut self.nick_input)
                    .hint_text("Ник")
                    .desired_width(w),
            );
            if r.changed() {
                self.nick_input = self
                    .nick_input
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
            }
            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.pass_input)
                    .hint_text("Пароль")
                    .password(true)
                    .desired_width(w),
            );
            ui.add_space(16.0);
            let can = !self.nick_input.is_empty() && !self.pass_input.is_empty();
            if ui
                .add_enabled(
                    can,
                    egui::Button::new(RichText::new("Войти").size(13.0).color(t.bg))
                        .fill(t.text)
                        .stroke(Stroke::NONE)
                        .rounding(8.0)
                        .min_size(vec2(w, 40.0)),
                )
                .clicked()
            {
                let nick = self.nick_input.trim().to_string();
                let hwid = format!(
                    "{:04X}-{:04X}",
                    self.rng.u64() % 0x10000,
                    self.rng.u64() % 0x10000
                );
                let plan = self
                    .keys
                    .get(self.selected_key)
                    .map(|k| k.plan.clone())
                    .unwrap_or_else(|| "Static".into());
                self.push_log(&format!("profile auth ok · {}", nick));
                self.toast_msg(&format!("Добро пожаловать, {}", nick));
                self.account = Some(Profile { nick, hwid, plan });
                self.pass_input.clear();
                self.modal = None;
            }
        }
    }

    fn unbind_modal(&mut self, ui: &mut egui::Ui, t: &Theme) {
        ui.label(RichText::new("Отвязать все ключи").size(18.0).strong().color(t.text));
        ui.add_space(8.0);
        let acc = self.account.clone();
        if let Some(p) = acc {
            ui.label(
                RichText::new("Все ключи потеряют привязку к железу")
                    .size(13.0)
                    .color(t.t2),
            );
            ui.label(
                RichText::new(format!("Введите ник «{}» для подтверждения", p.nick))
                    .size(12.0)
                    .color(t.t3),
            );
            ui.add_space(12.0);
            let w = ui.available_width();
            ui.add(
                egui::TextEdit::singleline(&mut self.unbind_input)
                    .hint_text(p.nick.clone())
                    .desired_width(w),
            );
            let ok = !p.nick.trim().is_empty()
                && self.unbind_input.trim().to_lowercase() == p.nick.trim().to_lowercase();
            ui.add_space(16.0);
            let btn = egui::Button::new(RichText::new("Отвязать все ключи").size(13.0).color(t.danger))
                .fill(t.s2)
                .stroke(Stroke::new(1.0, mix(t.danger, t.line2, 0.5)))
                .rounding(8.0)
                .min_size(vec2(w, 40.0));
            if ui.add_enabled(ok, btn).clicked() {
                for k in &mut self.keys {
                    k.bound = false;
                }
                self.push_log("hwid unbound · all keys");
                self.toast_msg("Все ключи отвязаны");
                self.modal = None;
            }
        } else {
            ui.label(
                RichText::new("Сначала войдите в аккаунт — подтверждение идёт по нику")
                    .size(13.0)
                    .color(t.t2),
            );
            ui.add_space(16.0);
            if ghost_button(ui, t, "Войти").clicked() {
                self.modal = Some(Modal::Auth);
            }
        }
    }

    fn play_error_modal(&mut self, ui: &mut egui::Ui, t: &Theme) {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
            let c = rect.center();
            ui.painter()
                .circle_stroke(c, 10.0, Stroke::new(1.5, t.danger));
            ui.painter().line_segment(
                [pos2(c.x, c.y - 5.0), pos2(c.x, c.y + 1.0)],
                Stroke::new(1.5, t.danger),
            );
            ui.painter().circle_filled(pos2(c.x, c.y + 5.0), 1.0, t.danger);
            ui.label(RichText::new("Ошибка запуска").size(18.0).strong().color(t.text));
        });
        ui.add_space(12.0);
        ui.label(
            RichText::new("game client not found. Try install 'A-Client'")
                .size(14.0)
                .color(t.t2),
        );
        ui.add_space(16.0);
        let w = ui.available_width();
        if ui
            .add(
                egui::Button::new(RichText::new("OK").size(13.0).color(t.bg))
                    .fill(t.text)
                    .stroke(Stroke::NONE)
                    .rounding(8.0)
                    .min_size(vec2(w, 40.0)),
            )
            .clicked()
        {
            self.modal = None;
        }
    }
}

impl eframe::App for LudoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let dt = if self.last_t < 0.0 {
            1.0 / 60.0
        } else {
            (now - self.last_t).clamp(0.001, 0.05)
        } as f32;
        self.last_t = now;
        let t = THEMES[self.theme_idx];
        apply_style(ctx, &t);

        if !self.deco_applied {
            self.deco_applied = true;
            if !self.custom_titlebar {
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.clone()));
            }
        }

        let s = self.sig();
        if s != self.last_sig {
            self.last_sig = s;
            self.dirty = true;
        }
        if self.dirty && now - self.last_save > 0.5 {
            self.save_cfg();
            self.dirty = false;
            self.last_save = now;
        }

        if self.buy_until > 0.0 && now >= self.buy_until {
            self.buy_until = 0.0;
            let plan = if self.buy_plan == 0 { "Static" } else { "Prime" }.to_string();
            let key = gen_key(&mut self.rng);
            self.keys.push(KeyEntry {
                plan,
                key: key.clone(),
                bound: false,
            });
            self.selected_key = self.keys.len() - 1;
            self.sync_plan();
            self.buy_result = Some(key);
            self.push_log("payment ok · key GG.…");
            self.toast_msg("Ключ получен");
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.key_menu.is_some() {
                self.key_menu = None;
            } else {
                self.modal = None;
            }
        }
        if self.modal != Some(Modal::Keys) {
            self.key_menu = None;
        }

        if now >= self.title_next {
            let mut idx = (self.rng.u64() % TITLES.len() as u64) as usize;
            if TITLES[idx] == self.title {
                idx = (idx + 1) % TITLES.len();
            }
            self.title = TITLES[idx].to_string();
            self.title_since = now;
            self.title_next = now + 24.0;
            if !self.custom_titlebar {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title.clone()));
            }
        }

        if self.custom_titlebar {
            egui::TopBottomPanel::top("titlebar")
                .exact_height(TITLEBAR_H)
                .frame(egui::Frame::none().fill(t.s2).inner_margin(Margin {
                    left: 12.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                }))
                .show(ctx, |ui| {
                    let bar = ui.max_rect();
                    let drag = ui.interact(bar, ui.id().with("tb_drag"), Sense::click_and_drag());
                    if drag.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    if drag.double_clicked() {
                        self.maximized = !self.maximized;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                    }
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(
                            self.tex_neon.id(),
                            vec2(16.0, 16.0),
                        )));
                        ui.label(RichText::new("ludoclient").monospace().size(12.0).color(t.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            if caption_btn(ui, &t, 2, self.maximized) {
                                self.save_cfg();
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if caption_btn(ui, &t, 1, self.maximized) {
                                self.maximized = !self.maximized;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                            }
                            if caption_btn(ui, &t, 0, self.maximized) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        });
                    });
                    let a = if self.smooth_anim {
                        ((now - self.title_since) / 0.35).clamp(0.0, 1.0) as f32
                    } else {
                        1.0
                    };
                    ui.painter().text(
                        bar.center(),
                        Align2::CENTER_CENTER,
                        self.title.as_str(),
                        FontId::monospace(11.0),
                        mix(Color32::TRANSPARENT, t.t2, a),
                    );
                });
        }

        egui::SidePanel::right("settings")
            .exact_width(300.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(t.s1).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.client_section(ui, &t);
                        self.custom_section(ui, &t, ctx);
                        self.other_section(ui, &t);
                        self.account_section(ui, &t, ctx);
                        ui.add_space(16.0);
                    });
            });

        self.banstat_panel(ctx, &t, dt);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(t.bg))
            .show(ctx, |ui| {
                let bounds = ui.max_rect().shrink(2.0);
                self.last_bounds = bounds;

                // задний план: частицы рисуются ДО виджетов, поэтому они "сзади"
                let bg_painter = ui.painter().clone();
                self.particles_frame(&bg_painter, bounds, dt, now);

                if !self.toys_seeded && bounds.width() > 100.0 {
                    for k in 0..5 {
                        self.spawn_toy(bounds, 1);
                        if let Some(toy) = self.toys.last_mut() {
                            toy.pos.y -= k as f32 * 110.0;
                        }
                    }
                    self.toys_seeded = true;
                    self.next_spawn = now + 9.0;
                }

                let mut obstacles: Vec<Rect> = Vec::new();
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space((bounds.height() * 0.13).max(20.0));
                    let play = ui.add(
                        egui::Button::new(RichText::new("Играть").size(20.0).strong().color(t.bg))
                            .fill(t.text)
                            .stroke(Stroke::NONE)
                            .rounding(12.0)
                            .min_size(vec2(240.0, 64.0)),
                    );
                    if play.clicked() {
                        self.push_log("launch failed · game client not found");
                        self.modal = Some(Modal::PlayError);
                    }
                    obstacles.push(play.rect);
                    ui.add_space(12.0);
                    let row = ui.allocate_ui_with_layout(
                        vec2(212.0, 24.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.label(RichText::new("Инжектировать чит").size(13.0).color(t.t2));
                            switch(ui, &t, &mut self.inject_cheat, self.smooth_anim);
                        },
                    );
                    obstacles.push(row.response.rect);
                    ui.add_space(24.0);

                    let loader = egui::Frame::none()
                        .fill(t.s1)
                        .stroke(Stroke::new(1.0, t.line))
                        .rounding(16.0)
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_width(380.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("ludoclient · loader v2.1")
                                        .monospace()
                                        .size(12.0)
                                        .color(t.t2),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new("idle").monospace().size(11.0).color(t.t3));
                                });
                            });
                            ui.add_space(4.0);
                            for ac in ["paperplate", "neoac", "nohack", "valcuria"] {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(ac).monospace().size(12.0).color(t.text));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new("undetected")
                                                .monospace()
                                                .size(11.0)
                                                .color(t.accent),
                                        );
                                        let (dot, _) =
                                            ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                        ui.painter().circle_filled(dot.center(), 3.0, t.accent);
                                    });
                                });
                            }
                            ui.add_space(8.0);
                            egui::Frame::none()
                                .fill(t.bg)
                                .stroke(Stroke::new(1.0, t.line))
                                .rounding(8.0)
                                .inner_margin(Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.set_width(356.0);
                                    ui.set_min_height(116.0);
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    let skip = self.log.len().saturating_sub(6);
                                    for l in self.log.iter().skip(skip) {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&l.time)
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(t.accent_dim),
                                            );
                                            ui.label(
                                                RichText::new(&l.msg)
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(t.t2),
                                            );
                                        });
                                    }
                                });
                        });
                    obstacles.push(loader.response.rect);
                });

                ui.painter().text(
                    pos2(bounds.left() + 12.0, bounds.bottom() - 8.0),
                    Align2::LEFT_BOTTOM,
                    "© 2026 LudoClient · все инструменты — для тестирования систем",
                    FontId::monospace(10.0),
                    t.t3,
                );

                if self.toast.as_ref().map_or(false, |(_, u)| now >= *u) {
                    self.toast = None;
                }
                if let Some((msg, until)) = self.toast.clone() {
                    let a = ((until - now) / 0.5).clamp(0.0, 1.0) as f32;
                    ui.painter().text(
                        pos2(bounds.center().x, bounds.top() + 20.0),
                        Align2::CENTER_CENTER,
                        msg,
                        FontId::proportional(13.0),
                        mix(Color32::TRANSPARENT, t.accent, a),
                    );
                }

                self.toys_frame(ui, bounds, &obstacles, dt, now);
            });

        if self.modal.is_some() {
            let screen = ctx.screen_rect();
            let top = if self.custom_titlebar {
                screen.top() + TITLEBAR_H
            } else {
                screen.top()
            };
            let dim_rect = Rect::from_min_max(pos2(screen.left(), top), screen.max);
            let dim = egui::Area::new(egui::Id::new("modal_dim"))
                .order(egui::Order::Middle)
                .fixed_pos(dim_rect.min)
                .movable(false)
                .interactable(true)
                .show(ctx, |ui| {
                    let (rect, resp) = ui.allocate_exact_size(dim_rect.size(), Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(6, 6, 8, 184),
                    );
                    resp
                })
                .inner;
            if dim.clicked() {
                self.modal = None;
                self.key_menu = None;
            }
        }
        if let Some(m) = self.modal {
            let width = match m {
                Modal::Keys => 420.0,
                Modal::Buy => 440.0,
                Modal::Auth => 360.0,
                Modal::Unbind => 400.0,
                Modal::PlayError => 420.0,
            };
            egui::Area::new(egui::Id::new("modal_card"))
                .order(egui::Order::Foreground)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .movable(false)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(t.s1)
                        .stroke(Stroke::new(1.0, t.line2))
                        .rounding(16.0)
                        .inner_margin(Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_width(width);
                            match m {
                                Modal::Keys => self.keys_modal(ui, &t, ctx),
                                Modal::Buy => self.buy_modal(ui, &t, ctx, now),
                                Modal::Auth => self.auth_modal(ui, &t),
                                Modal::Unbind => self.unbind_modal(ui, &t),
                                Modal::PlayError => self.play_error_modal(ui, &t),
                            }
                        });
                });
        }

        if let Some((i, pos)) = self.key_menu {
            let scr = ctx.screen_rect();
            let pos = pos2(
                pos.x.min(scr.right() - 176.0).max(scr.left()),
                pos.y.min(scr.bottom() - 48.0).max(scr.top()),
            );
            let mut act = false;
            egui::Area::new(egui::Id::new("key_ctx"))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos)
                .movable(false)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(t.s2)
                        .stroke(Stroke::new(1.0, t.line2))
                        .rounding(8.0)
                        .inner_margin(Margin::same(4.0))
                        .show(ui, |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Удалить ключ").size(13.0).color(t.danger),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .rounding(6.0)
                                    .min_size(vec2(160.0, 32.0)),
                                )
                                .clicked()
                            {
                                act = true;
                            }
                        });
                });
            if act {
                if i < self.keys.len() {
                    self.keys.remove(i);
                    if self.selected_key >= self.keys.len() {
                        self.selected_key = self.keys.len().saturating_sub(1);
                    }
                    self.sync_plan();
                    self.push_log("key deleted");
                    self.toast_msg("Ключ удалён");
                }
                self.key_menu = None;
            } else if ctx.input(|i| i.pointer.primary_clicked()) {
                self.key_menu = None;
            }
        }

        if self.custom_titlebar && !self.maximized {
            resize_grips(ctx);
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([1080.0, 680.0])
        .with_decorations(false)
        .with_resizable(true)
        .with_title("LudoClient");
    if let Ok(img) = image::open("assets/neon.png").or_else(|_| image::open("neon.png")) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        viewport = viewport.with_icon(egui::IconData {
            rgba: rgba.into_raw(),
            width: w,
            height: h,
        });
    }
    let opts = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "LudoClient",
        opts,
        Box::new(|cc| Ok(Box::new(LudoApp::new(cc)))),
    )
}