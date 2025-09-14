use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static HISTORY_REFRESH: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
}

pub fn set_history_refresh(cb: Option<Rc<dyn Fn()>>) {
    HISTORY_REFRESH.with(|cell| {
        *cell.borrow_mut() = cb;
    });
}

pub fn emit_history_refresh() {
    HISTORY_REFRESH.with(|cell| {
        if let Some(cb) = cell.borrow().as_ref() {
            cb();
        }
    });
}
