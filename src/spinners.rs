use std::{process::Command, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{CommandInfo, Target};

const TICK_RATE: Duration = Duration::from_millis(30);

#[derive(Clone)]
pub struct TargetSpinner {
    inner: ProgressBar,
    target: Target,
}

impl TargetSpinner {
    pub fn with_target(target: Target) -> Self {
        let spinner_style =
            ProgressStyle::with_template(" {spinner:.bold.dim}   {wide_msg}").unwrap();

        let inner = ProgressBar::new_spinner()
            .with_style(spinner_style)
            .with_message(format!("Building target {}...", target.display_name()));
        inner.enable_steady_tick(TICK_RATE);

        Self { inner, target }
    }

    pub fn finish(&self) {
        let spinner_finish_style =
            ProgressStyle::with_template("{prefix:.bold.green} {wide_msg:.bold}").unwrap();

        self.inner.set_style(spinner_finish_style.clone());
        self.inner.set_prefix("DONE");
        self.inner.finish_with_message(format!(
            "Successfully built target {}",
            self.target.display_name()
        ))
    }
}

impl From<TargetSpinner> for ProgressBar {
    fn from(outer: TargetSpinner) -> Self {
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
