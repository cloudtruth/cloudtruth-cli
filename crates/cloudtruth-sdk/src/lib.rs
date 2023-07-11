use std::sync::Arc;
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
    akvpulltask_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pulls_tasks_steps_retrieve() {}
}
pub struct steps {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
    akvpulltask_pk: &str,
}
impl steps {
    pub fn integrations_azure_key_vault_pulls_tasks_steps_list() {}
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
            id,
        }
    }
}
pub struct akvpulltask_pk {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
    akvpulltask_pk: &str,
}
impl akvpulltask_pk {
    pub fn steps(&self) -> steps {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        } = self;
        steps {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        }
    }
}
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pulls_tasks_retrieve() {}
}
pub struct tasks {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
}
impl tasks {
    pub fn integrations_azure_key_vault_pulls_tasks_list() {}
    pub fn akvpulltask_pk(&self, akvpulltask_pk: &str) -> akvpulltask_pk {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        akvpulltask_pk {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        }
    }
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            akvpull_pk,
            id,
        }
    }
}
pub struct akvpull_pk {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpull_pk: &str,
}
impl akvpull_pk {
    pub fn tasks(&self) -> tasks {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        tasks {
            client,
            akvintegration_pk,
            akvpull_pk,
        }
    }
}
pub struct sync {
    client: Arc<Client>,
    akvintegration_pk: &str,
    id: &str,
}
impl sync {
    pub fn integrations_azure_key_vault_pulls_sync_create() {}
}
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pulls_retrieve() {}
    pub fn integrations_azure_key_vault_pulls_update() {}
    pub fn integrations_azure_key_vault_pulls_destroy() {}
    pub fn integrations_azure_key_vault_pulls_partial_update() {}
    pub fn sync(&self) -> sync {
        let Self {
            client,
            akvintegration_pk,
            id,
        } = self;
        sync {
            client,
            akvintegration_pk,
            id,
        }
    }
}
pub struct pulls {
    client: Arc<Client>,
    akvintegration_pk: &str,
}
impl pulls {
    pub fn integrations_azure_key_vault_pulls_list() {}
    pub fn integrations_azure_key_vault_pulls_create() {}
    pub fn akvpull_pk(&self, akvpull_pk: &str) -> akvpull_pk {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        akvpull_pk {
            client,
            akvintegration_pk,
            akvpull_pk,
        }
    }
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            id,
        }
    }
}
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
    akvpushtask_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pushes_tasks_steps_retrieve() {}
}
pub struct steps {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
    akvpushtask_pk: &str,
}
impl steps {
    pub fn integrations_azure_key_vault_pushes_tasks_steps_list() {}
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
            id,
        }
    }
}
pub struct akvpushtask_pk {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
    akvpushtask_pk: &str,
}
impl akvpushtask_pk {
    pub fn steps(&self) -> steps {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        } = self;
        steps {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        }
    }
}
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pushes_tasks_retrieve() {}
}
pub struct tasks {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
}
impl tasks {
    pub fn integrations_azure_key_vault_pushes_tasks_list() {}
    pub fn akvpushtask_pk(&self, akvpushtask_pk: &str) -> akvpushtask_pk {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        akvpushtask_pk {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        }
    }
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            akvpush_pk,
            id,
        }
    }
}
pub struct akvpush_pk {
    client: Arc<Client>,
    akvintegration_pk: &str,
    akvpush_pk: &str,
}
impl akvpush_pk {
    pub fn tasks(&self) -> tasks {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        tasks {
            client,
            akvintegration_pk,
            akvpush_pk,
        }
    }
}
pub struct sync {
    client: Arc<Client>,
    akvintegration_pk: &str,
    id: &str,
}
impl sync {
    pub fn integrations_azure_key_vault_pushes_sync_create() {}
}
pub struct id {
    client: Arc<Client>,
    akvintegration_pk: &str,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_pushes_retrieve() {}
    pub fn integrations_azure_key_vault_pushes_update() {}
    pub fn integrations_azure_key_vault_pushes_destroy() {}
    pub fn integrations_azure_key_vault_pushes_partial_update() {}
    pub fn sync(&self) -> sync {
        let Self {
            client,
            akvintegration_pk,
            id,
        } = self;
        sync {
            client,
            akvintegration_pk,
            id,
        }
    }
}
pub struct pushes {
    client: Arc<Client>,
    akvintegration_pk: &str,
}
impl pushes {
    pub fn integrations_azure_key_vault_pushes_list() {}
    pub fn integrations_azure_key_vault_pushes_create() {}
    pub fn akvpush_pk(&self, akvpush_pk: &str) -> akvpush_pk {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        akvpush_pk {
            client,
            akvintegration_pk,
            akvpush_pk,
        }
    }
    pub fn id(&self, id: &str) -> id {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        id {
            client,
            akvintegration_pk,
            id,
        }
    }
}
pub struct akvintegration_pk {
    client: Arc<Client>,
    akvintegration_pk: &str,
}
impl akvintegration_pk {
    pub fn pulls(&self) -> pulls {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        pulls {
            client,
            akvintegration_pk,
        }
    }
    pub fn pushes(&self) -> pushes {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        pushes {
            client,
            akvintegration_pk,
        }
    }
}
pub struct CloudtruthSdk {
    client: Arc<Client>,
}
impl CloudtruthSdk {
    fn new() -> Self {
        CloudtruthSdk {
            client: Arc::new(Client::new()),
        }
    }
    pub fn instance() -> &'static Self {
        static ONCE: OnceCell<CloudtruthSdk> = OnceCell::new();
        ONCE.get_or_init(CloudtruthSdk::new)
    }
    pub fn integrations(&self) -> integrations {
        let Self { client } = self;
        integrations { client }
    }
}
pub struct integrations {
    client: Arc<Client>,
}
impl integrations {
    pub fn azure(&self) -> azure {
        let Self { client } = self;
        azure { client }
    }
}
pub struct azure {
    client: Arc<Client>,
}
impl azure {
    pub fn key_vault(&self) -> key_vault {
        let Self { client } = self;
        key_vault { client }
    }
}
pub struct key_vault {
    client: Arc<Client>,
}
impl key_vault {
    pub fn integrations_azure_key_vault_list() {}
    pub fn integrations_azure_key_vault_create() {}
    pub fn akvintegration_pk(&self, akvintegration_pk: &str) -> akvintegration_pk {
        let Self { client } = self;
        akvintegration_pk {
            client,
            akvintegration_pk,
        }
    }
    pub fn id(&self, id: &str) -> id {
        let Self { client } = self;
        id { client, id }
    }
}
pub struct id {
    client: Arc<Client>,
    id: &str,
}
impl id {
    pub fn integrations_azure_key_vault_retrieve() {}
    pub fn integrations_azure_key_vault_update() {}
    pub fn integrations_azure_key_vault_destroy() {}
    pub fn integrations_azure_key_vault_partial_update() {}
    pub fn scan(&self) -> scan {
        let Self { client, id } = self;
        scan { client, id }
    }
}
pub struct scan {
    client: Arc<Client>,
    id: &str,
}
impl scan {
    pub fn integrations_azure_key_vault_scan_create() {}
}
