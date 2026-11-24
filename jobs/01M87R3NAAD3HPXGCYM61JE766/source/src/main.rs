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
    Theme { name: "Прайм", bg: rgb(0x0e0c14), s1: rgb(0x15121d), s2: rgb(0x1a1724), s3: rgb