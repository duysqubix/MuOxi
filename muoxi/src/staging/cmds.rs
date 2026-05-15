#![allow(missing_docs)]

//! Definitions and collections of all commands found throughout MuOxi
//! whether in the staging server or the game itself.

use crate::comms::Client;
use crate::prelude::{Command, CommandResult};
use crate::send;
use async_trait::async_trait;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

pub async fn do_cmd(
    client: &mut Client,
    cmd: Option<&mut (dyn Command + Send)>,
    errmsg: &str,
) -> CommandResult<()> {
    if let Some(cmd) = cmd {
        cmd.execute_cmd(client).await?;
    } else {
        send(client, errmsg).await.unwrap();
    }
    Ok(())
}

pub mod proxy_commands {
    use super::*;

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct CmdProxyNew;

    #[async_trait]
    impl Command for CmdProxyNew {
        fn name(&self) -> &str {
            "new"
        }

        fn aliases(&self) -> Vec<&str> {
            vec!["n"]
        }

        async fn execute_cmd(&self, _client: &mut Client) -> CommandResult<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct CmdProxyAccount;

    #[async_trait]
    impl Command for CmdProxyAccount {
        fn name(&self) -> &str {
            "account"
        }

        fn aliases(&self) -> Vec<&str> {
            vec!["acc"]
        }

        async fn execute_cmd(&self, _client: &mut Client) -> CommandResult<()> {
            Ok(())
        }
    }
}

#[allow(dead_code)]
mod game_commands {}
