fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        if let Err(e) = res.compile() {
            eprintln!("Ошибка компиляции ресурсов иконки: {}", e);
        }
    }
}