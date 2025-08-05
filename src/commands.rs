use std::{collections::VecDeque, fmt::Error};

use uuid::Uuid;

use crate::shotliner_document::{Tag, TagID};
use crate::production;
pub enum CommandHistoryStatus {
    ExecuteSuccess,
    UndoSuccess,
    UndoLimitReached,
    RedoLimitReached,
}

pub struct CommandHistory {
    pub index: u64,
    pub history: VecDeque<Command>,
    pub max_history_size: u64,
    pub current_history_size: u64,
}
impl CommandHistory {
    fn execute(&mut self) -> Result<CommandHistoryStatus, Error> {
        if self.index < self.max_history_size {
            self.index = self.index + 1;
            return Ok(CommandHistoryStatus::ExecuteSuccess);
        } else {
            return Err(Error);
        }
    }
    fn undo(&mut self) {
        if self.index > 0 {
            self.index = self.index - 1;
        } else {
            // TODO:
        }
    }
    fn redo(&mut self) {
        if self.index + 1 <= self.current_history_size {
            self.index = self.index + 1;
        }
    }
}

pub enum Command {
    AddShotline(production::ShotID, production::Shot),
    ModifyShotline(production::ShotID, Option<production::Shot>), // takes ID, old shotline
    RemoveShotline(production::ShotID, Option<production::Shot>), // takes ID, old shotline

    AddTag(TagID, Tag),
    ModifyTag(TagID, Option<Tag>),
    RemoveTag(TagID, Option<Tag>),
}
