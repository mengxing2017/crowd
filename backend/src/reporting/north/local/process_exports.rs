use crate::reporting::south::port::Repository;
use crate::shared::Result;
#[derive(Clone)]
pub struct ProcessExports {
    repo: Repository,
}
impl ProcessExports {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(&self) -> Result<()> {
        let mut tx = self.repo.begin().await?;
        let mut s = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        if crate::reporting::domain::process_exports(&mut s, now) > 0 {
            s.log(
                "report-worker",
                "export_complete",
                "reporting",
                "export-worker",
                &Repository::time(&mut tx).await?,
            );
            Repository::save(&mut tx, &s).await?;
        }
        Repository::commit(tx).await
    }
}
