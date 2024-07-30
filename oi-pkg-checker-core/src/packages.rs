pub mod components;
mod cycles;
mod de_serialization;
pub mod depend_types;
pub mod dependency_type;
pub mod package;
pub mod rev_depend_type;

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! shared_type {
    ($t:ty) => {
        std::rc::Rc<std::cell::RefCell<$t>>
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! weak_type {
    ($t:ty) => {
        std::rc::Weak<std::cell::RefCell<$t>>
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! new {
    ($t:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($t))
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! clone {
    ($t:expr) => {
        std::rc::Rc::clone($t)
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! downgrade {
    ($t:expr) => {
        std::rc::Rc::downgrade($t)
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! get {
    ($shared:expr) => {
        $shared.borrow()
    };
}

#[macro_export]
#[cfg(not(feature = "thread_safe"))]
macro_rules! get_mut {
    ($shared:expr) => {
        $shared.borrow_mut()
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! shared_type {
    ($t:ty) => {
        std::sync::Arc<std::sync::Mutex<$t>>
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! weak_type {
    ($t:ty) => {
        std::sync::Weak<std::sync::Mutex<$t>>
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! new {
    ($t:expr) => {
        std::sync::Arc::new(std::sync::Mutex::new($t))
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! clone {
    ($t:expr) => {
        std::sync::Arc::clone($t)
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! downgrade {
    ($t:expr) => {
        std::sync::Arc::downgrade($t)
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! get {
    ($shared:expr) => {
        $shared.lock().unwrap()
    };
}

#[macro_export]
#[cfg(feature = "thread_safe")]
macro_rules! get_mut {
    ($shared:expr) => {
        get!($shared)
    };
}
