#![cfg_attr(target_family = "wasm", no_main)]

mod product_ui;
mod scene;

use std::borrow::Cow;

use anyhow::Result;
use gpui_platform::application;
use ochub_ui::gpui::{
    App, AppContext, AssetSource, Bounds, SharedString, WindowBounds, WindowOptions, px, size,
};
use ochub_ui::{assets, i18n, theme};
use scene::{SceneConfig, SceneView};

struct SceneAssets;

impl AssetSource for SceneAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(assets::load(path))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(assets::list(path))
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    let config = SceneConfig::from_browser();
    i18n::install_tag(&config.locale);

    let handle = application()
        .with_assets(SceneAssets)
        .run_embedded(move |cx: &mut App| {
            ochub_ui::install(cx);
            theme::install_family(
                &theme::ochub_family(),
                theme::ThemeMode::System,
                cx.window_appearance(),
            );

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.), px(800.)),
                        cx,
                    ))),
                    window_min_size: Some(size(px(320.), px(360.))),
                    window_background: theme::window_background_appearance(),
                    ..Default::default()
                },
                |_window, cx| cx.new(|_| SceneView::new(config.clone())),
            )
            .expect("open OcHub documentation scene");
            cx.activate(true);
        });
    // The browser owns the run loop and tears down the entire WASM instance on
    // navigation, so this page-scoped application must outlive `start`.
    std::mem::forget(handle);
}
