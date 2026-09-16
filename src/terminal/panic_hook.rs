//! Process-wide panic dispatch with prompt-thread-local cleanup.

use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Once},
};

use super::Restoration;

// Stable Rust cannot conditionally restore a hook without overwriting a later owner.
// Keep one dispatcher installed and add/remove only thread-local session handlers.
static INSTALL_DISPATCHER: Once = Once::new();
type Handler = Arc<dyn Fn() + Send + Sync>;

thread_local! {
    static ACTIVE_HANDLER: RefCell<Option<Handler>> = const { RefCell::new(None) };
}

pub(super) struct Registration {
    previous: Option<Handler>,
    _owning_thread: PhantomData<Rc<()>>,
}

pub(super) fn register(restoration: Arc<Restoration>) -> Registration {
    install_dispatcher();
    register_handler(Arc::new(move || {
        let _ = restoration.restore();
    }))
}

fn register_handler(handler: Handler) -> Registration {
    let previous = ACTIVE_HANDLER.with(|active| active.replace(Some(handler)));
    Registration {
        previous,
        _owning_thread: PhantomData,
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        ACTIVE_HANDLER.with(|active| {
            active.replace(self.previous.take());
        });
    }
}

fn install_dispatcher() {
    INSTALL_DISPATCHER.call_once(|| {
        let chained = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            dispatch(run_active_handler, || chained(information));
        }));
    });
}

fn run_active_handler() {
    ACTIVE_HANDLER.with(|active| {
        if let Ok(active) = active.try_borrow()
            && let Some(handler) = active.as_ref()
        {
            handler();
        }
    });
}

fn dispatch(restore: impl FnOnce(), chained: impl FnOnce()) {
    restore();
    chained();
}
