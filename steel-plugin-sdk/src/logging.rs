#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::error(&message);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::warn(&message);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::info(&message);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::debug(&message);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::trace(&message);
    };
}
