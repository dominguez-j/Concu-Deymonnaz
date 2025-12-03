use crate::enterprise::Enterprise;
use crate::messages::AdminCommand;
use actix::Addr;
use lib::messages::types::UpdateType;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug)]
enum ParseCommandError {
    Empty,
    UnknownCommand(String),
    InvalidArguments(String),
}

pub struct AdminInput;

impl AdminInput {
    pub fn new(enterprise_addr: Addr<Enterprise>) {
        let mut reader = BufReader::new(tokio::io::stdin());
        actix::spawn(async move {
            loop {
                let input = Self::read_input(&mut reader).await;
                match Self::parse_command(&input) {
                    Ok(command) => enterprise_addr.do_send(command),
                    Err(ParseCommandError::Empty) => continue,
                    Err(ParseCommandError::UnknownCommand(cmd)) => {
                        eprintln!("Error: Unknown command '{}'", cmd)
                    }
                    Err(ParseCommandError::InvalidArguments(msg)) => {
                        eprintln!("Error: Invalid arguments - {}", msg)
                    }
                };
            }
        });
    }

    async fn read_input(reader: &mut BufReader<tokio::io::Stdin>) -> String {
        let mut buf = String::new();
        if reader.read_line(&mut buf).await.unwrap_or(0) == 0 {
            return String::new();
        }
        buf.trim().to_string()
    }

    fn parse_update_type(s: &str) -> Result<UpdateType, ParseCommandError> {
        match s {
            "add" => Ok(UpdateType::Increment),
            "sub" => Ok(UpdateType::Decrement),
            "set" => Ok(UpdateType::Set),
            _ => Err(ParseCommandError::InvalidArguments(format!(
                "Invalid update type '{}'",
                s
            ))),
        }
    }

    fn parse_command(input: &str) -> Result<AdminCommand, ParseCommandError> {
        if input.is_empty() {
            return Err(ParseCommandError::Empty);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.as_slice() {
            ["enterprise", "view"] => Ok(AdminCommand::EnterpriseView {}),
            ["card", "view", id_str] => {
                let card_id = id_str.parse::<u32>().map_err(|_| {
                    ParseCommandError::InvalidArguments("Card ID must be a number.".to_string())
                })?;
                Ok(AdminCommand::CardView { card_id })
            }
            ["card", op, id_str, limit_str] => {
                let update_type = Self::parse_update_type(op)?;
                let card_id = id_str.parse::<u32>().map_err(|_| {
                    ParseCommandError::InvalidArguments("Card ID must be a number.".to_string())
                })?;
                let limit = limit_str.parse::<u32>().map_err(|_| {
                    ParseCommandError::InvalidArguments(
                        "Limit must be a positive number.".to_string(),
                    )
                })?;
                Ok(AdminCommand::UpdateCardLimit {
                    card_id,
                    limit,
                    update_type,
                })
            }
            ["enterprise", op, limit_str] => {
                let update_type = Self::parse_update_type(op)?;
                let limit = limit_str.parse::<u32>().map_err(|_| {
                    ParseCommandError::InvalidArguments(
                        "Limit must be a positive number.".to_string(),
                    )
                })?;
                Ok(AdminCommand::UpdateEnterpriseLimit { limit, update_type })
            }
            _ => Err(ParseCommandError::UnknownCommand(input.to_string())),
        }
    }
}
