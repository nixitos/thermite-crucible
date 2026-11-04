use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;
use image::GenericImageView; // Необходимо для вызова .dimensions()

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    // 1. Загрузка иконки для окна (из PNG)
    // Убедитесь, что файл icon.png лежит в корне проекта
    let icon_image = image::open("icon.png").expect("Не удалось найти или открыть icon.png");
    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_rgba8().into_raw();
    let window_icon = tao::window::Icon::from_rgba(rgba, width, height)
        .expect("Не удалось создать иконку из RGBA данных");

    // 2. Создание окна с иконкой
    let window = WindowBuilder::new()
        .with_title("DeepSeek WebView")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_window_icon(Some(window_icon))
        .build(&event_loop)
        .expect("Не удалось создать окно");

    // 3. JavaScript для фикса Alt и установки масштаба 80%
    let init_script = r#"
        (function() {
            // === ФИКС ALT ===
            document.addEventListener('keydown', function(e) {
                if (e.key === 'Alt' && e.shiftKey) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[ALT_FIX] Shift+Alt подавлен');
                }
            }, true); // Фаза погружения (срабатывает раньше всех)

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
            // Ждем загрузки DOM, чтобы не применить зум к пустой странице
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', applyZoom);
            } else {
                applyZoom();
            }
            console.log('[ALT_FIX] JavaScript-обработчик установлен');
        })();
    "#;

    // 4. Создание WebView
    let _webview = WebViewBuilder::new()
        .with_url("https://chat.deepseek.com")
        .with_data_directory("./deepseek_user_data") // Сохраняет cookies и localStorage
        .with_initialization_script(init_script)     // Внедряет JS на каждой странице
        .build(&window)
        .expect("Не удалось создать WebView");

    // 5. Запуск цикла событий
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