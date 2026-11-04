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

    // 1. Иконка окна
    let icon_image = image::open("icon.png").expect("Не удалось найти или открыть icon.png");
    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_rgba8().into_raw();
    let window_icon = tao::window::Icon::from_rgba(rgba, width, height)
        .expect("Не удалось создать иконку из RGBA данных");

    // 2. Окно
    let window = WindowBuilder::new()
        .with_title("DeepSeek WebView")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_window_icon(Some(window_icon))
        .build(&event_loop)
        .expect("Не удалось создать окно");

    // 3. Контекст для сохранения cookies и localStorage
    let data_dir = PathBuf::from("./deepseek_user_data");
    let mut web_context = WebContext::new(Some(data_dir));

    // 4. Скрипт (Alt-фикс + зум 80%)
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

    // 5. Создание WebView через статический with_web_context
    let _webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_url("https://chat.deepseek.com")
        .with_initialization_script(init_script)
        .build(&window)
        .expect("Не удалось создать WebView");

    // 6. Цикл событий
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