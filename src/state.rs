use dioxus::prelude::*;

// Estado global que cualquier componente puede usar
pub fn use_app_state() -> Signal<&'static str> {
    use_context::<Signal<&'static str>>()
}

pub fn provide_app_state() {
    use_context_provider(|| Signal::new("home"));
}