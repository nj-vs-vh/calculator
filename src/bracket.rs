use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketType {
    Round,
    Curly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketSide {
    Opening,
    Closing,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bracket {
    pub type_: BracketType,
    pub side: BracketSide,
}

impl Debug for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.type_, self.side)
    }
}

pub struct BracketStack {
    stack: Vec<BracketType>,
}

impl BracketStack {
    pub fn new() -> BracketStack {
        BracketStack { stack: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn update(&mut self, bracket: Bracket) -> Result<(), String> {
        match bracket.side {
            BracketSide::Opening => {
                self.stack.push(bracket.type_);
                Ok(())
            }
            BracketSide::Closing => {
                if self.stack.is_empty() || self.stack[self.stack.len() - 1] != bracket.type_ {
                    Err("unmatched closing bracket".into())
                } else {
                    self.stack.pop();
                    Ok(())
                }
            }
        }
    }
}
