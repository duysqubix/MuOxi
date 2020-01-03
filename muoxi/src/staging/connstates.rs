use machine::{machine, methods, transitions};

machine! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum ConnState{
        AwaitingAcctName{name: String},
        NewAcctName{name: String},
        MainMenu,
        EnterGame
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Next;

#[derive(Clone, Debug, PartialEq)]
pub struct NewAcct {
    name: String,
}

impl NewAcct {
    pub fn new(name: String) -> Self {
        Self { name: name }
    }
}

transitions!(ConnState,
  [
    (AwaitingAcctName, Next) => MainMenu,
    (AwaitingAcctName, NewAcct) => NewAcctName,
    (MainMenu, Next) => EnterGame,
    (EnterGame, Next) => MainMenu
  ]
);

methods!(ConnState,
    [
        AwaitingAcctName => get name: String,
        AwaitingAcctName => set name: String
    ]
);

impl NewAcctName {
    pub fn on_next(self, _: Next) -> MainMenu {
        MainMenu {}
    }
}

impl AwaitingAcctName {
    pub fn on_next(self, _: Next) -> MainMenu {
        MainMenu {}
    }

    pub fn on_new_acct(self, newacct: NewAcct) -> NewAcctName {
        NewAcctName { name: newacct.name }
    }

    pub fn new() -> Self {
        Self {
            name: "".to_string(),
        }
    }
}

impl MainMenu {
    pub fn on_next(self, _: Next) -> EnterGame {
        EnterGame {}
    }
}

impl EnterGame {
    pub fn on_next(self, _: Next) -> MainMenu {
        MainMenu {}
    }
}

// #[cfg(test)]

// mod tests {
//     use super::*;

//     #[test]
//     fn state_transitions() {
//         let mut state = ConnState::AwaitingAcctName(AwaitingAcctName {
//             val: "".to_string(),
//         });
//         assert_eq!(state.val(), "".to_string());
//         println!("{:?} {:?}", state, state.val());
//         if let Some(v) = state.val_mut() {
//             *v = "Duan Uys".to_string();
//         }
//         assert_eq!(state.val(), "Duan Uys".to_string());

//         state = state.on_next(Next);
//     }
// }
