use std::path::PathBuf;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder};

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("DeepSeek WebView")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .unwrap();

    // 1. Создаем контекст с директорией для сохранения cookies и localStorage
    let mut web_context = WebContext::new(Some(PathBuf::from("./deepseek_user_data")));

    // JavaScript для фикса Alt и установки масштаба 80%
    let init_script = r#"
        (function() {
            // === ФИКС ALT ===
            document.addEventListener('keydown', function(e) {
                if (e.key === 'Alt' && e.shiftKey) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[ALT_FIX] Shift+Alt подавлен в JavaScript');
                }
            }, true);

            document.addEventListener('keyup', function(e) {
                if (e.key === 'Alt') {
                    e.preventDefault();
                    e.stopPropagation();
                }
            }, true);

            // === МАСШТАБ 80% ===
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
            console.log('[ALT_FIX] JavaScript-обработчик Alt установлен');
        })();
    "#;

    // 2. Передаем контекст в WebViewBuilder
    let _webview = WebViewBuilder::new()
        .with_web_context(&mut web_context) // <-- Привязываем контекст сохранения сессии
        .with_url("https://chat.deepseek.com")
        .with_initialization_script(init_script)
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}