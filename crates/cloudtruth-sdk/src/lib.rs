use once_cell::sync::OnceCell;
use reqwest::blocking::Client;
use std::sync::Arc;
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkStepsId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
    akvpulltask_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkStepsId {
    pub fn retrieve() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkSteps {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
    akvpulltask_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkSteps {
    pub fn list() {}
    pub fn id(
        &self,
        id: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkStepsId {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkStepsId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.clone(),
            akvpulltask_pk: akvpulltask_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPk {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
    akvpulltask_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPk {
    pub fn steps(
        &self,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkSteps {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
            akvpulltask_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPkSteps {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.clone(),
            akvpulltask_pk: akvpulltask_pk.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksId {
    pub fn retrieve() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasks {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasks {
    pub fn list() {}
    pub fn akvpulltask_pk(
        &self,
        akvpulltask_pk: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPk {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksAkvpulltaskPk {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.clone(),
            akvpulltask_pk: akvpulltask_pk.into(),
        }
    }
    pub fn id(
        &self,
        id: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksId {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasksId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPk {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpull_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPk {
    pub fn tasks(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasks {
        let Self {
            client,
            akvintegration_pk,
            akvpull_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPkTasks {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsIdSync {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsIdSync {
    pub fn create() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPullsId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPullsId {
    pub fn retrieve() {}
    pub fn update() {}
    pub fn destroy() {}
    pub fn partial_update() {}
    pub fn sync(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsIdSync {
        let Self {
            client,
            akvintegration_pk,
            id,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsIdSync {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            id: id.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPulls {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPulls {
    pub fn list() {}
    pub fn create() {}
    pub fn akvpull_pk(
        &self,
        akvpull_pk: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPk {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsAkvpullPk {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpull_pk: akvpull_pk.into(),
        }
    }
    pub fn id(&self, id: impl Into<Arc<str>>) -> IntegrationsAzureKeyVaultAkvintegrationPkPullsId {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPullsId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkStepsId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
    akvpushtask_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkStepsId {
    pub fn retrieve() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkSteps {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
    akvpushtask_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkSteps {
    pub fn list() {}
    pub fn id(
        &self,
        id: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkStepsId {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkStepsId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.clone(),
            akvpushtask_pk: akvpushtask_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPk {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
    akvpushtask_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPk {
    pub fn steps(
        &self,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkSteps {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
            akvpushtask_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPkSteps {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.clone(),
            akvpushtask_pk: akvpushtask_pk.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksId {
    pub fn retrieve() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasks {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasks {
    pub fn list() {}
    pub fn akvpushtask_pk(
        &self,
        akvpushtask_pk: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPk {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksAkvpushtaskPk {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.clone(),
            akvpushtask_pk: akvpushtask_pk.into(),
        }
    }
    pub fn id(
        &self,
        id: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksId {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasksId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPk {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    akvpush_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPk {
    pub fn tasks(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasks {
        let Self {
            client,
            akvintegration_pk,
            akvpush_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPkTasks {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesIdSync {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesIdSync {
    pub fn create() {}
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushesId {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushesId {
    pub fn retrieve() {}
    pub fn update() {}
    pub fn destroy() {}
    pub fn partial_update() {}
    pub fn sync(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesIdSync {
        let Self {
            client,
            akvintegration_pk,
            id,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesIdSync {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            id: id.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPkPushes {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPkPushes {
    pub fn list() {}
    pub fn create() {}
    pub fn akvpush_pk(
        &self,
        akvpush_pk: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPk {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesAkvpushPk {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            akvpush_pk: akvpush_pk.into(),
        }
    }
    pub fn id(&self, id: impl Into<Arc<str>>) -> IntegrationsAzureKeyVaultAkvintegrationPkPushesId {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushesId {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultAkvintegrationPk {
    client: Arc<Client>,
    akvintegration_pk: Arc<str>,
}
impl IntegrationsAzureKeyVaultAkvintegrationPk {
    pub fn pulls(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPulls {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPulls {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
        }
    }
    pub fn pushes(&self) -> IntegrationsAzureKeyVaultAkvintegrationPkPushes {
        let Self {
            client,
            akvintegration_pk,
        } = self;
        IntegrationsAzureKeyVaultAkvintegrationPkPushes {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.clone(),
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
    pub fn integrations(&self) -> Integrations {
        let Self { client } = self;
        Integrations {
            client: client.clone(),
        }
    }
}
pub struct Integrations {
    client: Arc<Client>,
}
impl Integrations {
    pub fn azure(&self) -> IntegrationsAzure {
        let Self { client } = self;
        IntegrationsAzure {
            client: client.clone(),
        }
    }
}
pub struct IntegrationsAzure {
    client: Arc<Client>,
}
impl IntegrationsAzure {
    pub fn key_vault(&self) -> IntegrationsAzureKeyVault {
        let Self { client } = self;
        IntegrationsAzureKeyVault {
            client: client.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVault {
    client: Arc<Client>,
}
impl IntegrationsAzureKeyVault {
    pub fn list() {}
    pub fn create() {}
    pub fn akvintegration_pk(
        &self,
        akvintegration_pk: impl Into<Arc<str>>,
    ) -> IntegrationsAzureKeyVaultAkvintegrationPk {
        let Self { client } = self;
        IntegrationsAzureKeyVaultAkvintegrationPk {
            client: client.clone(),
            akvintegration_pk: akvintegration_pk.into(),
        }
    }
    pub fn id(&self, id: impl Into<Arc<str>>) -> IntegrationsAzureKeyVaultId {
        let Self { client } = self;
        IntegrationsAzureKeyVaultId {
            client: client.clone(),
            id: id.into(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultId {
    client: Arc<Client>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultId {
    pub fn retrieve() {}
    pub fn update() {}
    pub fn destroy() {}
    pub fn partial_update() {}
    pub fn scan(&self) -> IntegrationsAzureKeyVaultIdScan {
        let Self { client, id } = self;
        IntegrationsAzureKeyVaultIdScan {
            client: client.clone(),
            id: id.clone(),
        }
    }
}
pub struct IntegrationsAzureKeyVaultIdScan {
    client: Arc<Client>,
    id: Arc<str>,
}
impl IntegrationsAzureKeyVaultIdScan {
    pub fn create() {}
}
