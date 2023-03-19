use std::ops::Not;

use crate::spinners::{MainSpinner, Ticking};
use crate::{Config, Result};

pub fn run_step<T, E, S>(config: &Config, title: S, execute: E) -> Result<T>
where
    E: FnOnce() -> Result<T>,
    S: ToString,
{
    let spinner = config
        .silent
        .not()
        .then_some(MainSpinner::with_message(title.to_string()));

    spinner.start();

    let result = execute();

    match result {
        Ok(_) => spinner.finish(),
        Err(_) => spinner.fail(),
    }

    result
}
