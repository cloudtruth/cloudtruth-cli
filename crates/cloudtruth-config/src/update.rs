use crate::config_date::naive_date_serde;
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum Action {
    Warn,
    Error,
    Update,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(default)]
pub struct Updates {
    pub check: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(skip_serializing_if = "Option::is_none", with = "naive_date_serde")]
    pub last_checked: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Frequency>,
}

impl Frequency {
    pub fn days(&self) -> i64 {
        match self {
            Self::Daily => 1,
            Self::Weekly => 7,
            Self::Monthly => 30,
        }
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::Weekly
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Warn
    }
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            check: true,
            action: None,
            last_checked: None,
            frequency: None,
        }
    }
}

impl Updates {
    pub fn next_update(&self) -> Option<NaiveDate> {
        if !self.check {
            return None;
        }
        let last = self
            .last_checked
            .or_else(|| NaiveDate::from_ymd_opt(2021, 6, 1))
            .unwrap();
        let freq = self.frequency.unwrap_or_default();
        let delta = Duration::days(freq.days());
        last.checked_add_signed(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn default_frequency() {
        assert_eq!(Frequency::default(), Frequency::Weekly)
    }

    #[test]
    fn frequency_days() {
        assert_eq!(Frequency::Daily.days(), 1);
        assert_eq!(Frequency::Weekly.days(), 7);
        assert_eq!(Frequency::Monthly.days(), 30);
    }

    #[test]
    fn default_action() {
        assert_eq!(Action::default(), Action::Warn);
    }

    #[test]
    fn default_updates() {
        let default = Updates::default();
        assert!(default.check);
        assert_eq!(default.last_checked, None);
        assert_eq!(default.action, None);
        assert_eq!(default.frequency, None);

        assert_eq!(default.frequency.unwrap_or_default(), Frequency::Weekly);
        assert_eq!(default.action.unwrap_or_default(), Action::Warn);
    }

    #[test]
    fn update_next_update() {
        let mut update = Updates::default();

        // bogus default that always triggers an update
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2021, 6, 8).unwrap()
        );

        // change frequency and see differences
        update.frequency = Some(Frequency::Daily);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2021, 6, 2).unwrap()
        );
        update.frequency = Some(Frequency::Weekly);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2021, 6, 8).unwrap()
        );
        update.frequency = Some(Frequency::Monthly);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2021, 7, 1).unwrap()
        );

        // with checking disabled, a None is returned
        update.check = false;
        assert_eq!(update.next_update(), None);
        update.check = true;

        // different date with 31 days in the month (even in the future!)
        update.last_checked = Some(NaiveDate::from_ymd_opt(2050, 3, 1)).unwrap();
        update.frequency = Some(Frequency::Daily);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2050, 3, 2).unwrap()
        );
        update.frequency = Some(Frequency::Weekly);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2050, 3, 8).unwrap()
        );
        update.frequency = Some(Frequency::Monthly);
        assert_eq!(
            update.next_update().unwrap(),
            NaiveDate::from_ymd_opt(2050, 3, 31).unwrap()
        );
    }

    #[test]
    fn parse_update_ok() {
        let content = indoc!(
            r#"
        check: false
        "#
        );
        let update: Updates = serde_yaml::from_str(content).unwrap();
        assert!(!update.check);
        assert_eq!(update.action, None);
        assert_eq!(update.frequency, None);
        assert_eq!(update.last_checked, None);

        let content = indoc!(
            r#"
        check: true
        action: Error
        frequency: Monthly
        "#
        );
        let update: Updates = serde_yaml::from_str(content).unwrap();
        assert!(update.check);
        assert_eq!(update.action.unwrap(), Action::Error);
        assert_eq!(update.frequency.unwrap(), Frequency::Monthly);
        assert_eq!(update.last_checked, None);

        let content = indoc!(
            r#"
        check: true
        action: Update
        frequency: Daily
        last_checked: 2021-1-1
        "#
        );
        //dbg!(serde_yaml::from_str::<Updates>(&content).unwrap_err());
        let update: Updates = serde_yaml::from_str(content).unwrap();
        assert!(update.check);
        assert_eq!(update.action.unwrap(), Action::Update);
        assert_eq!(update.frequency.unwrap(), Frequency::Daily);
        assert_eq!(
            update.last_checked.unwrap(),
            NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()
        );

        let content = indoc!(
            r#"
        check: true
        action: Warn
        frequency: Weekly
        last_checked: 2050-12-31
        "#
        );
        //dbg!(serde_yaml::from_str::<Updates>(&content).unwrap_err());
        let update: Updates = serde_yaml::from_str(content).unwrap();
        assert!(update.check);
        assert_eq!(update.action.unwrap(), Action::Warn);
        assert_eq!(update.frequency.unwrap(), Frequency::Weekly);
        assert_eq!(
            update.last_checked.unwrap(),
            NaiveDate::from_ymd_opt(2050, 12, 31).unwrap()
        );
    }

    #[test]
    fn parse_update_last_checked() {
        let content = indoc!(
            r#"
        check: true
        last_checked: DATE
        "#
        );
        let expected = NaiveDate::from_ymd_opt(2021, 4, 1).unwrap();
        for date in &["2021-04-01", "2021-4-1", "4-1-2021", "4/1/2021"] {
            let update: Updates = serde_yaml::from_str(&content.replace("DATE", date)).unwrap();
            assert_eq!(update.last_checked.unwrap(), expected);
        }

        let result = serde_yaml::from_str::<Updates>(content);
        assert!(result.is_err());

        let result = serde_yaml::from_str::<Updates>(&content.replace("DATE", "2020-20-20"));
        assert!(result.is_err());
    }
}
