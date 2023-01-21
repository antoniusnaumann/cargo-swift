use std::process::Command;

pub trait CommandInfo {
    fn info(&self) -> String;
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
}
