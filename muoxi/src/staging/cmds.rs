#![allow(missing_docs)]
// //!
// //! Definitions and collections of all commands found throughou MuOxi whether they
// //! be in staging server or game itself, this may eventually become it's own crate
// //!

use crate::comms::Client;
use crate::prelude::{CmdSet, Command, CommandResult};
use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

////*********************Proxy Staging Server Commands*************//
/// the command of 'new' to create a new account
pub mod proxy_commands {
    use super::*;
    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct CmdProxyNew;
    impl Command for CmdProxyNew {
        fn name(&self) -> &str {
            "new"
        }

        fn aliases(&self) -> Vec<&str> {
            vec!["n"]
        }

        fn execute_cmd(&self, client: &mut Client) -> CommandResult<()> {
            Ok(())
        }
    }

    /// command to connect to existing account
    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub struct CmdProxyAccount;
    impl Command for CmdProxyAccount {
        fn name(&self) -> &str {
            "account"
        }

        fn aliases(&self) -> Vec<&str> {
            vec!["acc"]
        }

        fn execute_cmd(&self, client: &mut Client) -> CommandResult<()> {
            Ok(())
        }
    }
}

mod GameCommands {
    //
    use super::*;
}
