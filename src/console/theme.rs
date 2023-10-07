use core::fmt;

use console::Style;
use dialoguer::theme::{ColorfulTheme, Theme};

pub fn prompt_theme() -> impl Theme {
    PromptTheme {
        theme: ColorfulTheme::default(),
        done: ColorfulTheme {
            prompt_style: Style::new().for_stderr(),
            ..ColorfulTheme::default()
        },
    }
}

pub struct PromptTheme<T: Theme> {
    /// The theme that should be used during the prompt
    theme: T,
    /// The theme that should be used after the prompt was finished
    done: T,
}

impl<T: Theme> Theme for PromptTheme<T> {
    /// Formats a prompt.
    #[inline]
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.theme.format_prompt(f, prompt)
    }

    /// Formats out an error.
    #[inline]
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        self.theme.format_error(f, err)
    }

    /// Formats a confirm prompt.
    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.theme.format_confirm_prompt(f, prompt, default)
    }

    /// Formats a confirm prompt after selection.
    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.done
            .format_confirm_prompt_selection(f, prompt, selection)
    }

    /// Formats an input prompt.
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.theme.format_input_prompt(f, prompt, default)
    }

    /// Formats an input prompt after selection.
    #[inline]
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.done.format_input_prompt_selection(f, prompt, sel)
    }

    /// Formats a select prompt.
    #[inline]
    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.theme.format_select_prompt(f, prompt)
    }

    /// Formats a select prompt after selection.
    #[inline]
    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.done.format_select_prompt_selection(f, prompt, sel)
    }

    /// Formats a multi select prompt.
    #[inline]
    fn format_multi_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    /// Formats a sort prompt.
    #[inline]
    fn format_sort_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    /// Formats a multi_select prompt after selection.
    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.done
            .format_multi_select_prompt_selection(f, prompt, selections)
    }

    /// Formats a sort prompt after selection.
    #[inline]
    fn format_sort_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.done
            .format_sort_prompt_selection(f, prompt, selections)
    }

    /// Formats a select prompt item.
    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        self.theme.format_select_prompt_item(f, text, active)
    }

    /// Formats a multi select prompt item.
    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        self.theme
            .format_multi_select_prompt_item(f, text, checked, active)
    }

    /// Formats a sort prompt item.
    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        self.theme.format_sort_prompt_item(f, text, picked, active)
    }
}
