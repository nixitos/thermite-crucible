#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder};
use image::GenericImageView;
use std::path::PathBuf;

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    // 1. Иконка окна из встроенных байтов (файл на диске при запуске не нужен)
    let icon_bytes = include_bytes!("../icon.png");
    let icon_image = image::load_from_memory(icon_bytes)
        .expect("Не удалось загрузить иконку из памяти");

    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_rgba8().into_raw();
    let window_icon = tao::window::Icon::from_rgba(rgba, width, height)
        .expect("Не удалось создать иконку из RGBA данных");

    // 2. Размер окна ≈ 1/4 площади экрана (половина по каждой оси).
    //    Делим физические пиксели на scale_factor(), чтобы на HiDPI
    //    окно не оказалось вдвое больше ожидаемого.
    let logical_size = match event_loop.primary_monitor() {
        Some(monitor) => {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            tao::dpi::LogicalSize::new(
                (size.width  as f64 / scale) * 0.5,
                (size.height as f64 / scale) * 0.5,
            )
        }
        None => tao::dpi::LogicalSize::new(960.0, 700.0),
    };

    // 3. Создание окна (resizable по умолчанию, но зададим явно)
    let window = WindowBuilder::new()
        .with_title("DeepSeek WebView")
        .with_inner_size(logical_size)
        .with_min_inner_size(tao::dpi::LogicalSize::new(600.0, 400.0))
        .with_resizable(true)
        .with_window_icon(Some(window_icon))
        .build(&event_loop)
        .expect("Не удалось создать окно");

    // 4. Контекст для сохранения cookies и localStorage
    let data_dir = PathBuf::from("./deepseek_user_data");
    let mut web_context = WebContext::new(Some(data_dir));

    // 5. Скрипт (Alt-фикс + зум 80%)
    let init_script = r#"
        (function() {
            document.addEventListener('keydown', function(e) {
                if (e.key === 'Alt' && e.shiftKey) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[ALT_FIX] Shift+Alt подавлен');
                }
            }, true);

            document.addEventListener('keyup', function(e) {
                if (e.key === 'Alt') {
                    e.preventDefault();
                    e.stopPropagation();
                }
            }, true);

            function applyZoom() {
                if (document.body) {
                    document.body.style.zoom = '0.8';
                    console.log('[ZOOM] Масштаб установлен на 80%');
                }
            }
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', applyZoom);
            } else {
                applyZoom();
            }
            console.log('[ALT_FIX] JavaScript-обработчик установлен');
        })();
    "#;

    // 6. Создание WebView
    let _webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_url("https://chat.deepseek.com")
        .with_initialization_script(init_script)
        .build(&window)
        .expect("Не удалось создать WebView");

    // 7. Цикл событий
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}