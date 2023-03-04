use std::{process::Command, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::targets::Target;
use crate::CommandInfo;

const TICK_RATE: Duration = Duration::from_millis(30);

fn main_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.bold.dim} {wide_msg}").unwrap()
}

fn main_spinner_finish_style() -> ProgressStyle {
    ProgressStyle::with_template("{prefix:.green} {wide_msg:.bold}").unwrap()
}

pub trait Spinner {
    fn spinner(self) -> ProgressBar;
}

pub trait Ticking {
    fn start(&self);
    fn finish(&self);
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

        Self { inner }
    }
}

impl Ticking for MainSpinner {
    fn start(&self) {
        self.inner.enable_steady_tick(TICK_RATE);
    }

    fn finish(&self) {
        let spinner_finish_style = main_spinner_finish_style();

        self.inner.set_style(spinner_finish_style);
        self.inner.set_prefix("✔");
        self.inner.finish();
    }
}

impl Ticking for Option<MainSpinner> {
    fn start(&self) {
        if let Some(this) = self {
            this.inner.enable_steady_tick(TICK_RATE);
        }
    }

    fn finish(&self) {
        if let Some(this) = self {
            this.finish()
        }
    }
}

impl Spinner for MainSpinner {
    fn spinner(self) -> ProgressBar {
        self.inner
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
            .with_style(spinner_style)
            .with_message(command.multiline_info(70).replace('\n', "\n\t\t"));

        Self { inner }
    }
}

impl Ticking for CommandSpinner {
    fn start(&self) {
        self.inner.enable_steady_tick(TICK_RATE);
    }

    fn finish(&self) {
        let spinner_finish_style = ProgressStyle::with_template("\t{msg:.dim}").unwrap();

        self.inner.set_style(spinner_finish_style);
        self.inner.finish();
    }
}

impl Ticking for Option<CommandSpinner> {
    fn start(&self) {
        if let Some(this) = self {
            this.inner.enable_steady_tick(TICK_RATE);
        }
    }

    fn finish(&self) {
        if let Some(this) = self {
            this.finish()
        }
    }
}

impl Spinner for CommandSpinner {
    fn spinner(self) -> ProgressBar {
        self.inner
    }
}

impl From<CommandSpinner> for ProgressBar {
    fn from(outer: CommandSpinner) -> Self {
        outer.inner
    }
}
pub trait OptionalMultiProgress {
    fn add(&self, progress: &Option<impl Spinner + Clone>);
}

impl OptionalMultiProgress for Option<MultiProgress> {
    fn add(&self, progress: &Option<impl Spinner + Clone>) {
        if let (Some(multi), Some(spinner)) = (self, progress) {
            multi.add(spinner.clone().spinner());
        }
    }
}
