use once_cell::sync::OnceCell;
use reqwest::Client;
pub struct CloudtruthSdk {
    pub client: Client,
}
impl CloudtruthSdk {
    fn new() -> CloudtruthSdk {
        CloudtruthSdk {
            client: Client::new(),
        }
    }
    pub fn instance() -> &'static CloudtruthSdk {
        static ONCE: OnceCell<CloudtruthSdk> = OnceCell::new();
        ONCE.get_or_init(CloudtruthSdk::new)
    }
}
