use anyhow::Result;
use itertools::Itertools;
use crate::resp::operation::Operation;


#[derive(Clone, Debug)]
pub struct Command {
    command: String,
    arguments: Vec<String>,
}

impl TryFrom<Operation> for Command {
    type Error = anyhow::Error;

    fn try_from(value: Operation) -> Result<Self> {
        let (command, args) = value.only_array()?;
        let args: Vec<String> = args.into_iter()
            .map(|arg| arg.only_bulk())
            .try_collect()?;

        Ok(Command {
            command,
            arguments: args,
        })
    }
}

impl Command {
    pub fn as_str(&self) -> &str {
        self.command.as_str()
    }

    pub fn single_argument(&self) -> Result<String> {
        self.arguments
            .first()
            .ok_or(anyhow::anyhow!(format!("Invalid arguments: {:?}", self.arguments.iter().map(|arg| arg.to_string()).join(", "))))
            .map(|str| str.clone())
    }

    pub fn first_2_arguments(&self) -> Result<(String, String)> {
        let arguments: Option<(String, String)> = self.arguments
            .clone()
            .into_iter()
            .next_tuple();

        arguments.ok_or(anyhow::anyhow!(format!("Invalid arguments: {:?}", self.arguments.iter().map(|arg| arg.to_string()).join(", "))))
    }

    pub fn optional_arguments_after(&self, n: usize) -> Vec<String> {
        self.arguments
            .clone()
            .into_iter()
            .skip(n)
            .collect()
    }
}