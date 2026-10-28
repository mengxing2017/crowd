use crate::files::south::port::Repository;
use crate::shared::{Failure, Result};

#[derive(Clone)]
pub struct ReadPrivateFile {
    repo: Repository,
}

impl ReadPrivateFile {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, token: &str, id: &str, request: &str) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("FORBIDDEN", "账号不可查看私有资料"));
        }
        let result = Repository::file(&mut tx, id, &actor).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(&actor, "file_view", id, request, &now);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
}
