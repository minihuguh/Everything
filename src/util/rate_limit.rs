use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[derive(Clone)]
pub struct RateLimitedSender {
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    timeout_id: Option<i32>,
    interval_ms: f64,
    last_fire_ms: f64,
    pending: Option<f64>,
    callback: Box<dyn Fn(f64)>,
}

impl RateLimitedSender {
    pub fn new<F>(callback: F, interval_ms: f64) -> Self
    where
        F: Fn(f64) + 'static,
    {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                timeout_id: None,
                interval_ms,
                last_fire_ms: f64::NEG_INFINITY,
                pending: None,
                callback: Box::new(callback),
            })),
        }
    }

    pub fn send(&self, value: f64) {
        let now = js_sys::Date::now();
        let mut inner = self.inner.borrow_mut();

        if now - inner.last_fire_ms >= inner.interval_ms {
            if let Some(id) = inner.timeout_id.take() {
                let () = window().unwrap().clear_timeout_with_handle(id);
            }
            inner.pending = None;
            inner.last_fire_ms = now;
            drop(inner);
            (self.inner.borrow().callback)(value);
            return;
        }

        inner.pending = Some(value);
        if inner.timeout_id.is_none() {
            let wait = (inner.interval_ms - (now - inner.last_fire_ms)).max(0.0);
            let self_clone = self.clone();
            let closure = Closure::once_into_js(move || self_clone.fire_pending());
            let id = window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    wait as i32,
                )
                .unwrap();
            inner.timeout_id = Some(id);
        }
    }

    fn fire_pending(&self) {
        let value = {
            let mut inner = self.inner.borrow_mut();
            inner.timeout_id = None;
            match inner.pending.take() {
                Some(v) => {
                    inner.last_fire_ms = js_sys::Date::now();
                    v
                }
                None => return,
            }
        };
        (self.inner.borrow().callback)(value);
    }

    pub fn flush(&self, value: f64) {
        {
            let mut inner = self.inner.borrow_mut();
            if let Some(id) = inner.timeout_id.take() {
                let () = window().unwrap().clear_timeout_with_handle(id);
            }
            inner.pending = None;
            inner.last_fire_ms = js_sys::Date::now();
        }
        (self.inner.borrow().callback)(value);
    }
}
