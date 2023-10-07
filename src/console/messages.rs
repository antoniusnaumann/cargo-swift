use super::Config;

use console::{style, Style};

/// Prints a formatted warning message to the console
pub fn warn(msg: &str, config: &Config) {
    let style = Style::new().bold().yellow();
    print_msg("!", msg, style, config)
}

#[inline(always)]
fn print_msg(tag: &str, msg: &str, s: Style, config: &Config) {
    if !config.silent {
        println!("{} {}", s.apply_to(tag), style(msg))
    }
}

// TODO: macro to combine with format string
