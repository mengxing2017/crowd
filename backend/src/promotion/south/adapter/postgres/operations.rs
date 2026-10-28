use crate::platform::south::port::{Repository, database};
use crate::promotion::domain::Code;
use crate::shared::{Failure, Result};

impl Repository {
    pub async fn process_code_job(&self, user_id: &str) -> Result<()> {
        let mut tx = self.begin().await?;
        let mut state = Self::load(&mut tx).await?;
        state.user(user_id)?;
        if !state
            .codes
            .values()
            .any(|c| c.owner == user_id && c.active && c.product.is_none())
        {
            let mut candidate = Self::entropy(&mut tx).await?;
            while state.codes.contains_key(&candidate) {
                candidate = Self::entropy(&mut tx).await?;
            }
            state.codes.insert(
                candidate.clone(),
                Code {
                    owner: user_id.into(),
                    version: 1,
                    active: true,
                    product: None,
                    task: None,
                },
            );
            let now = Self::time(&mut tx).await?;
            state.log(
                user_id,
                "REQ-04-001",
                &candidate,
                "code-generation-job",
                &now,
            );
        }
        sqlx::query(
            "UPDATE crowd_code_jobs SET status='completed',attempts=attempts+1 WHERE user_id=$1",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        Self::save(&mut tx, &state).await?;
        Self::commit(tx).await
    }
    pub async fn retry_code_jobs(&self) -> Result<()> {
        let ids:Vec<String>=sqlx::query_scalar("SELECT user_id FROM crowd_code_jobs WHERE status='pending' ORDER BY created_at LIMIT 32").fetch_all(&self.pool).await.map_err(database)?;
        for id in ids {
            if self.process_code_job(&id).await.is_err() {
                tracing::warn!("promotion code generation remains pending");
            }
        }
        Ok(())
    }
}

impl Repository {
    pub(crate) async fn save_code_ownership(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        state: &crate::platform::south::port::rules::State,
    ) -> Result<()> {
        let inactive: Vec<String> = state
            .codes
            .iter()
            .filter(|(_, c)| !c.active)
            .map(|(id, _)| id.clone())
            .collect();
        sqlx::query("UPDATE crowd_code_ownership SET active=false WHERE code=ANY($1)")
            .bind(&inactive)
            .execute(&mut **tx)
            .await
            .map_err(database)?;
        for (id, code) in &state.codes {
            let result=sqlx::query("INSERT INTO crowd_code_ownership(code,owner_id,product_id,task_id,active,version) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(code) DO UPDATE SET active=EXCLUDED.active WHERE crowd_code_ownership.owner_id=EXCLUDED.owner_id AND crowd_code_ownership.product_id IS NOT DISTINCT FROM EXCLUDED.product_id AND crowd_code_ownership.task_id IS NOT DISTINCT FROM EXCLUDED.task_id")
                .bind(id).bind(&code.owner).bind(&code.product).bind(&code.task).bind(code.active).bind(code.version as i64).execute(&mut **tx).await.map_err(database)?;
            if result.rows_affected() != 1 {
                return Err(Failure("CODE_COLLISION", "推广码已被其他对象使用"));
            }
        }
        Ok(())
    }
}
