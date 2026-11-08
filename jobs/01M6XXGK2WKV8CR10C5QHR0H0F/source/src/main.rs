#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use tao::{
    dpi::{LogicalSize, PhysicalSize}, // Добавляем PhysicalSize
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};
use wry::{WebContext, WebViewBuilder};

const ICON_BYTES: &[u8] = include_bytes!("../icon.png");

fn load_window_icon() -> Option<Icon> {
    let img = image::load_from_memory(ICON_BYTES).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let mut window_builder = WindowBuilder::new().with_title("DeepSeek");
    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    let window = window_builder.build(&event_loop).unwrap();

    // --- ИСПРАВЛЕНИЕ 1: Корректный расчет размера окна с учетом DPI ---
    let monitor = window.current_monitor().unwrap();
    let scale_factor = monitor.scale_factor();
    let physical_size = monitor.size();
    
    // Переводим физический размер монитора в логические пиксели и берем половину
    let logical_size = physical_size.to_logical::<f64>(scale_factor);
    let half_width = logical_size.width / 2.0;
    let half_height = logical_size.height / 2.0;
    
    window.set_inner_size(LogicalSize::new(half_width, half_height));
    // -----------------------------------------------------------------

    let data_directory = PathBuf::from(r"D:\webviews");
    let mut web_context = WebContext::new(Some(data_directory));

    // --- ИСПРАВЛЕНИЕ 2: Надежная инъекция цветов через MutationObserver ---
    let init_script = r#"
        (function() {
            // --- ALT-FIX ---
            document.addEventListener('keydown', function(e) {
                if (e.key === 'Alt' && e.shiftKey) {
                    e.preventDefault();
                    e.stopPropagation();
                }
            }, true);
            document.addEventListener('keyup', function(e) {
                if (e.key === 'Alt') {
                    e.preventDefault();
                    e.stopPropagation();
                }
            }, true);

            // --- ЦВЕТА ---
            const colors = {
                '--dsw-alias-brand-primary': '#f1b57f',
                '--dsw-alias-button-ghost-active-border': '#d9a668',
                '--dsw-alias-bg-base': '#0e0e0e',
                '--dsw-alias-brand-text': '#df9a5d',
                '--dsw-alias-label-caption': '#69635b',
                '--dsw-alias-button-ghost-active-fill': '#433e3c',
                '--dsw-alias-button-ghost-active-hover': '#675c57',
                '--dsw-alias-label-tertiary': '#d1b06a',
                '--dsw-alias-button-elevated-fill': '#735739',
                '--dsw-alias-button-floating-fill': '#3e3a36',
                '--dsw-alias-brand-primary-invert': '#f1b588'
            };

            function applyColors() {
                const root = document.documentElement;
                if (!root) return;
                for (const [name, value] of Object.entries(colors)) {
                    root.style.setProperty(name, value, 'important'); // Важно: !important
                }
                let style = document.getElementById('ds-custom-style');
                if (!style) {
                    style = document.createElement('style');
                    style.id = 'ds-custom-style';
                    style.textContent = `.ds-think-content .ds-markdown { color: #afafaf !important; }`;
                    (document.head || document.documentElement).appendChild(style);
                }
            }

            // Применяем сразу и следим за изменениями в DOM
            applyColors();
            const observer = new MutationObserver(applyColors);
            observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true });
        })();
    "#;
    // --------------------------------------------------------------------

    let webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_url("https://chat.deepseek.com")
        .with_initialization_script(init_script)
        .with_hotkeys_zoom(false)
        .build(&window)?;

    webview.zoom(0.8)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}