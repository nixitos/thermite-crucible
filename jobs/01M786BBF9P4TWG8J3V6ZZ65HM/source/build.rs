use std::{fs::File, path::Path};

fn main() {
    // Пересобирать при изменении исходной иконки
    println!("cargo:rerun-if-changed=icon.png");

    // Только для Windows-таргета (работает и при кросс-компиляции)
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let png_path = Path::new("icon.png");
    let ico_path = Path::new("app.ico");

    // 1. Конвертируем PNG -> ICO
    if png_path.exists() {
        match image::open(png_path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();

                let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
                let icon_image = ico::IconImage::from_rgba_data(w, h, rgba.into_raw());

                match ico::IconDirEntry::encode(&icon_image) {
                    Ok(entry) => {
                        icon_dir.add_entry(entry);
                        match File::create(ico_path) {
                            Ok(f) => {
                                if let Err(e) = icon_dir.write(f) {
                                    eprintln!("Не удалось записать app.ico: {e}");
                                }
                            }
                            Err(e) => eprintln!("Не удалось создать app.ico: {e}"),
                        }
                    }
                    Err(e) => eprintln!("Не удалось закодировать ICO: {e}"),
                }
            }
            Err(e) => eprintln!("Не удалось открыть icon.png: {e}"),
        }
    } else {
        eprintln!("icon.png не найден в корне проекта — иконка будет стандартной");
    }

    // 2. Встраиваем ICO в ресурсы exe
    let mut res = winres::WindowsResource::new();
    if ico_path.exists() {
        res.set_icon("app.ico");
    }
    if let Err(e) = res.compile() {
        eprintln!("Ошибка компиляции ресурсов иконки: {e}");
    }
}