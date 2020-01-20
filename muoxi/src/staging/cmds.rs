#![allow(missing_docs)]
// //!
// //! Definitions and collections of all commands found throughou MuOxi whether they
// //! be in staging server or game itself, this may eventually become it's own crate
// //!

use crate::comms::Client;
use crate::prelude::{CmdSet, Command, CommandResult};
use crate::send;
use async_trait::async_trait;
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

pub async fn do_cmd<'a>(
    mut client: &mut Client,
    cmd: Option<&mut (dyn Command + Send)>,
    errmsg: &'a str,
) -> CommandResult<()> {
    if let Some(cmd) = cmd {
        cmd.execute_cmd(client).await?;
    } else {
        send(&mut client, errmsg).await.unwrap();
    }
    Ok(())
}

////*********************Proxy Staging Server Commands*************//
/// the command of 'new' to create a new account
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

    /// command to connect to existing account
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

mod game_commands {
    //
    // use super::*;
}
