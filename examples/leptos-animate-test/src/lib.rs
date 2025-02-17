// #![recursion_limit = "256"]

mod animated_for_page;
mod animated_layout_page;
mod animated_show_page;
mod animated_swap_page;
pub mod app;
mod dynamics_page;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
