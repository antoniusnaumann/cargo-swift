use std::{process::Command, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{CommandInfo, Target};

const TICK_RATE: Duration = Duration::from_millis(30);

fn main_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.bold.dim} {wide_msg}").unwrap()
}

fn main_spinner_finish_style() -> ProgressStyle {
    ProgressStyle::with_template("{prefix:.green} {wide_msg:.bold}").unwrap()
}

#[derive(Clone)]
pub struct MainSpinner {
    inner: ProgressBar,
}

impl MainSpinner {
    pub fn with_target(target: Target) -> Self {
        Self::with_message(format!("Building target {}...", target.display_name()))
    }

    pub fn with_message(msg: String) -> Self {
        let spinner_style = main_spinner_style();

        let inner = ProgressBar::new_spinner()
            .with_style(spinner_style)
            .with_message(msg);
        inner.enable_steady_tick(TICK_RATE);

        Self { inner }
    }

    pub fn finish(&self) {
        let spinner_finish_style = main_spinner_finish_style();

        self.inner.set_style(spinner_finish_style.clone());
        self.inner.set_prefix("✔");
        self.inner.finish();
    }
}

impl From<MainSpinner> for ProgressBar {
    fn from(outer: MainSpinner) -> Self {
        outer.inner
    }
}

#[derive(Clone)]
pub struct CommandSpinner {
    inner: ProgressBar,
}

impl CommandSpinner {
    pub fn with_command(command: &Command) -> Self {
        let spinner_style = ProgressStyle::with_template("\t{msg}").unwrap();

        let inner = ProgressBar::new_spinner()
            .with_style(spinner_style.clone())
            .with_message(command.info());

        Self { inner }
    }

    pub fn finish(&self) {
        let spinner_finish_style = ProgressStyle::with_template("\t{msg:.dim}").unwrap();

        self.inner.set_style(spinner_finish_style.clone());
        self.inner.finish();
    }
}

impl From<CommandSpinner> for ProgressBar {
    fn from(outer: CommandSpinner) -> Self {
        outer.inner
    }
}
