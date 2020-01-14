//!
//! Definitions and collections of all commands found throughou MuOxi whether they
//! be in staging server or game itself, this may eventually become it's own crate
//!

use crate::comms::Client;
use crate::prelude::{CmdSet, Command, CommandResult};

//*********************Proxy Staging Server Commands*************//
/// the command of 'new' to create a new account
pub struct NewCmd;
impl Command for NewCmd {
    fn name(&self) -> String {
        String::from("new")
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["n"]
    }

    fn execute_cmd(&self, client: &mut Client) -> CommandResult<()> {
        Ok(())
    }
}
