use std::any::Any;

pub fn panic_to_string(e: Box<dyn Any + Send>) -> String {
    let any = &*e;
    if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic occurred during shader compilation".to_string()
    }
}

