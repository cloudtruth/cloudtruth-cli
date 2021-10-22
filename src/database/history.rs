use cloudtruth_restapi::models::HistoryTypeEnum;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryAction {
    Create,
    Update,
    Delete,
    Nothing,
}

impl From<HistoryTypeEnum> for HistoryAction {
    fn from(action: HistoryTypeEnum) -> Self {
        match action {
            HistoryTypeEnum::Create => Self::Create,
            HistoryTypeEnum::Update => Self::Update,
            HistoryTypeEnum::Delete => Self::Delete,
            HistoryTypeEnum::Nothing => Self::Nothing,
        }
    }
}

impl fmt::Display for HistoryAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HistoryAction::Create => write!(f, "create"),
            HistoryAction::Update => write!(f, "update"),
            HistoryAction::Delete => write!(f, "delete"),
            HistoryAction::Nothing => write!(f, "nothing"),
        }
    }
}
