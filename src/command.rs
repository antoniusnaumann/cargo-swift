use std::process::Command;

pub trait CommandInfo {
    fn info(&self) -> String;
    fn multiline_info(&self, chars_per_line: usize) -> String;
}

impl CommandInfo for Command {
    fn info(&self) -> String {
        let program = self.get_program().to_string_lossy();
        let args = self
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        format!("{program} {args}")
    }

    /// Splits command info to multiple lines if it exceeds the given char count
    fn multiline_info(&self, chars_per_line: usize) -> String {
        let info = self.info();
        // This is not correct. However, since we are expecting only ASCII characters here, it should work anyways.
        if info.len() > chars_per_line {
            let program = self.get_program().to_string_lossy();
            // TODO: This is sloppy and does not deal with the possibility that a single argument could already be larger than chars_per_line
            let args = self
                .get_args()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" \\\n")
                .replace(" \\\n-", " -");

            format!("{program} {args}")
        } else {
            info
        }
    }
}
