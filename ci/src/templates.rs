use crate::config::{InstallType, ReleaseTest, TestOs};
use sailfish::TemplateOnce;

/// Template for generating the installation test Dockerfiles
#[derive(TemplateOnce)]
#[template(path = "Dockerfile.stpl")]
pub struct DockerTemplate<'conf> {
    pub os: TestOs,
    pub install_type: InstallType,
    pub version: &'conf str,
}

impl<'conf> DockerTemplate<'conf> {
    pub fn from_release_test(
        release_test: &'conf ReleaseTest,
    ) -> impl Iterator<Item = DockerTemplate<'conf>> {
        let &ReleaseTest {
            os,
            install_type,
            ref versions,
        } = release_test;
        versions.iter().map(move |version| DockerTemplate {
            os,
            install_type,
            version,
        })
    }
}
