use machine::{machine, methods, transitions};

machine! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum ConnState{
        AwaitingName{val: String},
        MainMenu,
        EnterGame
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Next;

transitions!(ConnState,
  [
    (AwaitingName, Next) => MainMenu,
    (MainMenu, Next) => EnterGame,
    (EnterGame, Next) => MainMenu
  ]
);

methods!(ConnState,
    [
        AwaitingName => get val: String,
        AwaitingName => set val: String
    ]
);
impl AwaitingName {
    pub fn on_next(self, _: Next) -> MainMenu {
        MainMenu {}
    }

    pub fn new() -> Self {
        Self {
            val: "".to_string(),
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

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn state_transitions() {
        let mut state = ConnState::AwaitingName(AwaitingName {
            val: "".to_string(),
        });
        assert_eq!(state.val(), "".to_string());
        println!("{:?} {:?}", state, state.val());
        if let Some(v) = state.val_mut() {
            *v = "Duan Uys".to_string();
        }
        assert_eq!(state.val(), "Duan Uys".to_string());

        state = state.on_next(Next);
    }
}
