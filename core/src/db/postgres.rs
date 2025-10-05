use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::info;
use uuid::Uuid;

use crate::{
    models::{
        channel_from_db, role_from_db, status_from_db, MintOutcome, MintRequest, MintStatus, Quota,
        Role, User,
    },
    repository::{
        DailyReportRow, MintRepository, QuotaRepository, ReportingRepository, UserRepository,
    },
};

const MAX_CONNECTIONS: u32 = 10;

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(url)
            .await?;

        Self::ensure_schema(&pool).await?;
        info!("postgres schema ready");

        Ok(Self { pool })
    }

    async fn ensure_schema(pool: &PgPool) -> Result<()> {
        let statements = [
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                channel TEXT NOT NULL,
                handle TEXT NOT NULL,
                role TEXT NOT NULL,
                domain TEXT NULL,
                last_seen_at TIMESTAMPTZ NOT NULL,
                UNIQUE(channel, handle)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS mint_requests (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                channel TEXT NOT NULL,
                amount BIGINT NOT NULL,
                status TEXT NOT NULL,
                tx_hash TEXT NULL,
                error TEXT NULL,
                requested_at TIMESTAMPTZ NOT NULL,
                processed_at TIMESTAMPTZ NULL,
                attempt INTEGER NOT NULL DEFAULT 0
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS mint_requests_requested_idx ON mint_requests(requested_at);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS quotas (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                day DATE NOT NULL,
                minted_total BIGINT NOT NULL,
                success_count BIGINT NOT NULL,
                UNIQUE(user_id, day)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS mint_failures (
                id UUID PRIMARY KEY,
                request_id UUID NOT NULL REFERENCES mint_requests(id),
                failed_at TIMESTAMPTZ NOT NULL,
                reason TEXT NOT NULL
            );
            "#,
        ];

        for statement in statements {
            sqlx::query(statement).execute(pool).await?;
        }

        Ok(())
    }

    fn map_user(row: &sqlx::postgres::PgRow) -> Result<User> {
        Ok(User {
            id: row.try_get("id")?,
            channel: channel_from_db(row.try_get::<&str, _>("channel")?)?,
            handle: row.try_get("handle")?,
            role: role_from_db(row.try_get::<&str, _>("role")?)?,
            domain: row.try_get("domain").ok(),
            last_seen_at: row.try_get("last_seen_at")?,
        })
    }

    fn map_request(row: &sqlx::postgres::PgRow) -> Result<MintRequest> {
        Ok(MintRequest {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            channel: channel_from_db(row.try_get::<&str, _>("channel")?)?,
            amount: row.try_get::<i64, _>("amount")? as u64,
            status: status_from_db(row.try_get::<&str, _>("status")?)?,
            tx_hash: row.try_get("tx_hash").ok(),
            error: row.try_get("error").ok(),
            requested_at: row.try_get("requested_at")?,
            processed_at: row.try_get("processed_at").ok(),
            attempt: row.try_get::<i32, _>("attempt")? as u16,
        })
    }

    fn map_quota(row: &sqlx::postgres::PgRow) -> Result<Quota> {
        Ok(Quota {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            day: row.try_get("day")?,
            minted_total: row.try_get::<i64, _>("minted_total")? as u64,
            success_count: row.try_get::<i64, _>("success_count")? as u64,
        })
    }

    fn map_report_row(row: &sqlx::postgres::PgRow) -> Result<DailyReportRow> {
        Ok(DailyReportRow {
            channel: row.try_get("channel")?,
            total_amount: row.try_get::<i64, _>("total_amount")? as u64,
            success_count: row.try_get::<i64, _>("success_count")? as u64,
            failure_count: row.try_get::<i64, _>("failure_count")? as u64,
        })
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresStore {
    async fn upsert_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, channel, handle, role, domain, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                channel = EXCLUDED.channel,
                handle = EXCLUDED.handle,
                role = EXCLUDED.role,
                domain = EXCLUDED.domain,
                last_seen_at = EXCLUDED.last_seen_at;
            "#,
        )
        .bind(user.id)
        .bind(user.channel.as_str())
        .bind(&user.handle)
        .bind(user.role.as_str())
        .bind(&user.domain)
        .bind(user.last_seen_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_user(&self, channel: &str, handle: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM users WHERE channel = $1 AND handle = $2 LIMIT 1
            "#,
        )
        .bind(channel)
        .bind(handle)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| Self::map_user(&r)).transpose()
    }

    async fn set_role(&self, user_id: Uuid, role: Role) -> Result<()> {
        sqlx::query(r#"UPDATE users SET role = $2 WHERE id = $1"#)
            .bind(user_id)
            .bind(role.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MintRepository for PostgresStore {
    async fn enqueue(&self, request: &MintRequest) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO mint_requests (id, user_id, channel, amount, status, tx_hash, error, requested_at, processed_at, attempt)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
                channel = EXCLUDED.channel,
                amount = EXCLUDED.amount,
                status = EXCLUDED.status,
                tx_hash = EXCLUDED.tx_hash,
                error = EXCLUDED.error,
                requested_at = EXCLUDED.requested_at,
                processed_at = EXCLUDED.processed_at,
                attempt = EXCLUDED.attempt;
            "#,
        )
        .bind(request.id)
        .bind(request.user_id)
        .bind(request.channel.as_str())
        .bind(request.amount as i64)
        .bind(request.status.as_str())
        .bind(&request.tx_hash)
        .bind(&request.error)
        .bind(request.requested_at)
        .bind(request.processed_at)
        .bind(request.attempt as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn next_pending(&self) -> Result<Option<MintRequest>> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT * FROM mint_requests
            WHERE status = 'pending'
            ORDER BY requested_at ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = row {
            let mut request = Self::map_request(&row)?;
            request.status = MintStatus::Processing;
            request.processed_at = Some(Utc::now());
            request.attempt += 1;

            sqlx::query(
                r#"
                UPDATE mint_requests
                SET status = $2, processed_at = $3, attempt = $4
                WHERE id = $1
                "#,
            )
            .bind(request.id)
            .bind(request.status.as_str())
            .bind(request.processed_at)
            .bind(request.attempt as i32)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(Some(request))
        } else {
            tx.rollback().await.ok();
            Ok(None)
        }
    }

    async fn update_status(&self, request_id: Uuid, status: MintStatus) -> Result<()> {
        let processed_at = match status {
            MintStatus::Completed | MintStatus::Failed => Some(Utc::now()),
            _ => None,
        };

        sqlx::query(
            r#"
            UPDATE mint_requests
            SET status = $2,
                processed_at = COALESCE($3, processed_at)
            WHERE id = $1
            "#,
        )
        .bind(request_id)
        .bind(status.as_str())
        .bind(processed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn record_outcome(&self, outcome: &MintOutcome) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE mint_requests
            SET status = $2,
                tx_hash = $3,
                error = $4,
                processed_at = $5,
                attempt = $6
            WHERE id = $1
            "#,
        )
        .bind(outcome.request.id)
        .bind(outcome.request.status.as_str())
        .bind(&outcome.tx_hash)
        .bind(&outcome.request.error)
        .bind(outcome.request.processed_at)
        .bind(outcome.request.attempt as i32)
        .execute(&self.pool)
        .await?;

        if outcome.request.status == MintStatus::Completed {
            sqlx::query(
                r#"
                UPDATE quotas
                SET success_count = success_count + 1
                WHERE user_id = $1 AND day = $2
                "#,
            )
            .bind(outcome.request.user_id)
            .bind(outcome.request.requested_at.date_naive())
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl QuotaRepository for PostgresStore {
    async fn record_mint(&self, user_id: Uuid, day: NaiveDate, amount: u64) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO quotas (id, user_id, day, minted_total, success_count)
            VALUES ($1, $2, $3, $4, 0)
            ON CONFLICT (user_id, day) DO UPDATE SET
                minted_total = quotas.minted_total + EXCLUDED.minted_total
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(day)
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn fetch_quota(&self, user_id: Uuid, day: NaiveDate) -> Result<Option<Quota>> {
        let row = sqlx::query(r#"SELECT * FROM quotas WHERE user_id = $1 AND day = $2"#)
            .bind(user_id)
            .bind(day)
            .fetch_optional(&self.pool)
            .await?;

        row.map(|r| Self::map_quota(&r)).transpose()
    }
}

#[async_trait::async_trait]
impl ReportingRepository for PostgresStore {
    async fn daily_summary(&self, day: NaiveDate) -> Result<Vec<DailyReportRow>> {
        let start = DateTime::<Utc>::from_utc(day.and_hms_opt(0, 0, 0).unwrap(), Utc);
        let end_date = day + Duration::days(1);
        let end = DateTime::<Utc>::from_utc(end_date.and_hms_opt(0, 0, 0).unwrap(), Utc);
        let rows = sqlx::query(
            r#"
            SELECT channel,
                   COALESCE(SUM(amount),0) AS total_amount,
                   SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) AS success_count,
                   SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS failure_count
            FROM mint_requests
            WHERE requested_at >= $1 AND requested_at < $2
            GROUP BY channel
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| Self::map_report_row(&row))
            .collect()
    }

    async fn log_failure(&self, request_id: Uuid, when: DateTime<Utc>, reason: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO mint_failures (id, request_id, failed_at, reason)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request_id)
        .bind(when)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
