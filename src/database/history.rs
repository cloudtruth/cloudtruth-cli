use cloudtruth_restapi::models::HistoryTypeEnum;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryAction {
    Create,
    Update,
    Delete,
    Nothing,
    Unknown,
}

impl From<HistoryTypeEnum> for HistoryAction {
    fn from(action: HistoryTypeEnum) -> Self {
        match action {
            HistoryTypeEnum::Create => Self::Create,
            HistoryTypeEnum::Update => Self::Update,
            HistoryTypeEnum::Delete => Self::Delete,
            HistoryTypeEnum::Nothing => Self::Nothing,
            HistoryTypeEnum::UnknownDefaultOpenApi => Self::Unknown,
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
            HistoryAction::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_to_history_type_enum() {
        // sanity check -- make sure we can successfully parse one item before testing the failures
        let result: serde_json::Result<HistoryTypeEnum> = serde_json::from_str("\"create\"");
        assert_eq!(HistoryTypeEnum::Create, result.unwrap());

        // checks that we fall back to the "unknown" value
        for item in vec!["foo", "bar"] {
            let value = format!("\"{}\"", item);
            let result: serde_json::Result<HistoryTypeEnum> = serde_json::from_str(&value);
            assert_eq!(false, result.is_err());
            assert_eq!(HistoryTypeEnum::UnknownDefaultOpenApi, result.unwrap());
        }
    }
}
