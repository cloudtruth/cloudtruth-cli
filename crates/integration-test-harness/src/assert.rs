use assert_cmd::assert::Assert;
use predicates::{prelude::PredicateBooleanExt, str::is_match};

/// An extension trait for custom assertion methods.
pub trait AssertCmdExt {
    fn paginated(self, page_size: usize) -> Self;
}

impl AssertCmdExt for Assert {
    fn paginated(self, page_size: usize) -> Self {
        let match_page =
            |n| is_match(format!(r"URL GET .+page={n}&page_size={page_size}")).unwrap();
        self.stdout(match_page(1).and(match_page(2)))
    }
}
