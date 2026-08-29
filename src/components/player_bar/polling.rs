use crate::ipc::commands;
use crate::state::PlayerState;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;

pub fn use_playback_polling(player: Signal<PlayerState>, active: Memo<bool>) {
    let interval_id: Rc<RefCell<Option<i32>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let id_for_effect = interval_id.clone();
    use_effect(move || {
        {
            let mut guard = id_for_effect.borrow_mut();
            if let Some(id) = guard.take() {
                let () = window().unwrap().clear_interval_with_handle(id);
            }
        }

        if !active() {
            return;
        }

        let closure = Closure::wrap(Box::new(move || {
            let mut player = player;
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(time) = commands::get_time().await {
                    player.with_mut(|p| {
                        p.current_time = time;
                        if p.duration > 0.0 && time >= p.duration - 1.0 {
                            p.advance();
                        }
                    });
                }
            });
        }) as Box<dyn FnMut()>);

        let id = window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            )
            .unwrap();
        *id_for_effect.borrow_mut() = Some(id);
        closure.forget();
    });

    let id_for_drop = interval_id;
    use_drop(move || {
        if let Some(id) = id_for_drop.borrow_mut().take()
            && let Some(w) = window()
        {
            let () = w.clear_interval_with_handle(id);
        }
    });
}
