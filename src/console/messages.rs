use super::Config;

use console::{style, Style};

/// Prints a formatted warning message to the console
pub fn print_warning(msg: &str, config: &Config) {
    let style = Style::new().bold().yellow();
    print_msg("!", msg, style, config)
}

/// Prints a formatted info message to the console
pub fn print_info(msg: &str, config: &Config) {
    let style = Style::new().bold().cyan();
    print_msg("ℹ", msg, style, config)
}

#[inline(always)]
fn print_msg(tag: &str, msg: &str, s: Style, config: &Config) {
    if !config.silent {
        println!("{} {}", s.apply_to(tag), style(msg))
    }
}

macro_rules! info {
    ($config:expr, $($arg:tt)*) => {
        $crate::console::messages::print_info(&format!($($arg)*), $config)
    };
}

macro_rules! warning {
    ($config:expr, $($arg:tt)*) => {
        $crate::console::messages::print_warning(&format!($($arg)*), $config)
    };
}

pub(crate) use info;
pub(crate) use warning;
