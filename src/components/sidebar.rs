use dioxus::prelude::*;
use crate::state::use_app_state;

fn home_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            path { d: "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
            polyline { points: "9 22 9 12 15 12 15 22" }
        }
    }
}

fn search_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

// fn lyrics_icon() -> Element {
//     rsx! {
//         svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
//             path { d: "M9 18V5l12-2v13" }
//             circle { cx: "6", cy: "18", r: "3" }
//             circle { cx: "18", cy: "16", r: "3" }
//         }
//     }
// }

fn stats_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            line { x1: "18", y1: "20", x2: "18", y2: "10" }
            line { x1: "12", y1: "20", x2: "12", y2: "4" }
            line { x1: "6", y1: "20", x2: "6", y2: "14" }
        }
    }
}

fn playlist_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        }
    }
}

fn artists_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            path { d: "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" }
            circle { cx: "12", cy: "7", r: "4" }
        }
    }
}

fn albums_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            polygon { points: "10 8 16 12 10 16 10 8" }
        }
    }
}

fn local_icon() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2", ry: "2" }
            line { x1: "3", y1: "9", x2: "21", y2: "9" }
            line { x1: "9", y1: "21", x2: "9", y2: "9" }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    let mut current_view = use_app_state();

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-logo", "Everything" }

            nav { class: "sidebar-nav",
                SidebarItem { current_view, view: "home", label: "Explorar", icon: home_icon() }
                SidebarItem { current_view, view: "search", label: "Buscar", icon: search_icon() }
                // SidebarItem { current_view, view: "lyrics", label: "Letras", icon: lyrics_icon() }
                SidebarItem { current_view, view: "stats", label: "Estadísticas", icon: stats_icon() }
            }

            div { class: "sidebar-divider" }

            div { class: "sidebar-section-title", "Biblioteca" }
            nav { class: "sidebar-nav",
                SidebarItem { current_view, view: "playlist", label: "Lista de reproducción", icon: playlist_icon() }
                SidebarItem { current_view, view: "artists", label: "Artistas", icon: artists_icon() }
                SidebarItem { current_view, view: "albums", label: "Álbumes", icon: albums_icon() }
                SidebarItem { current_view, view: "local", label: "Biblioteca local", icon: local_icon() }
            }
        }
    }
}

#[component]
fn SidebarItem(current_view: Signal<&'static str>, view: &'static str, label: &'static str, icon: Element) -> Element {
    let is_active = current_view() == view;

    rsx! {
        button {
            class: if is_active { "sidebar-item active" } else { "sidebar-item" },
            onclick: move |_| current_view.set(view),
            {icon}
            span { class: "sidebar-label", "{label}" }
        }
    }
}