// Windows 特定配置：隐藏控制台窗口
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod models;
mod dictionary;
mod gui;

use dictionary::Dictionary;
use eframe::egui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let dict = Dictionary::new("words".to_string());
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])  // 调整为更宽更长的窗口
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "DictNavi - English Dictionary",
        options,
        Box::new(move |cc| {
            // Start with the default fonts (we will be adding to them rather than replacing them).
            let mut fonts = egui::FontDefinitions::default();

            // Install my own font (maybe supporting non-latin characters).
            // .ttf and .otf files supported.
            fonts.font_data.insert(
                "NotoSans".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/NotoSansSC-Regular.ttf")).into(),
            );

            // Put my font first (highest priority) for proportional text:
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "NotoSans".to_owned());

            // Put my font as last fallback for monospace:
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("NotoSans".to_owned());

            // Tell egui to use these fonts:
            cc.egui_ctx.set_fonts(fonts);
            
            // 在闭包内部创建 app，并直接返回 Box
            let app = gui::DictNaviApp::new(dict);
            Box::new(app)
        }),
    )
}
