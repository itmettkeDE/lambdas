#[derive(Clone)]
pub struct CodeBuild {
    client: rusoto_codebuild::CodeBuildClient,
}

impl std::fmt::Debug for CodeBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBuild")
            .field("client", &"[...]")
            .finish()
    }
}

impl CodeBuild {
    pub(crate) fn new(region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_codebuild::CodeBuildClient::new(region),
        }
    }

    pub(crate) async fn get_projects(&self) -> anyhow::Result<Vec<String>> {
        use anyhow::Context;
        use rusoto_codebuild::CodeBuild;

        let mut token = None;
        let mut projects = Vec::new();
        loop {
            let res = self
                .client
                .list_projects(rusoto_codebuild::ListProjectsInput {
                    next_token: token.clone(),
                    ..rusoto_codebuild::ListProjectsInput::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch projects")?;
            if let Some(new_projects) = res.projects {
                projects.extend(new_projects);
            }
            if let Some(next_token) = res.next_token {
                token = Some(next_token);
                continue;
            }
            break;
        }
        Ok(projects)
    }
}
