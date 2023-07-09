use std::sync::Arc;

use once_cell::sync::OnceCell;
use reqwest::Client;

pub struct CloudtruthSdk {
    pub client: Arc<Client>,
}

impl CloudtruthSdk {
    fn new() -> CloudtruthSdk {
        CloudtruthSdk {
            client: Arc::new(Client::new()),
        }
    }
    pub fn instance() -> &'static CloudtruthSdk {
        static ONCE: OnceCell<CloudtruthSdk> = OnceCell::new();
        ONCE.get_or_init(CloudtruthSdk::new)
    }
}
