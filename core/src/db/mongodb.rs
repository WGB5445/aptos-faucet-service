use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
    options::{FindOneAndUpdateOptions, IndexOptions, ReturnDocument, UpdateOptions},
    Client, Collection, IndexModel,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    models::{
        channel_from_db, role_from_db, status_from_db, Channel, MintOutcome, MintRequest,
        MintStatus, Quota, Role, User,
    },
    repository::{
        DailyReportRow, MintRepository, QuotaRepository, ReportingRepository, UserRepository,
    },
};

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
    database: mongodb::Database,
}

impl MongoStore {
    pub async fn connect(url: &str, name: &str) -> Result<Self> {
        let client = Client::with_uri_str(url).await?;
        let database = client.database(name);
        let store = Self { client, database };
        store.ensure_indexes().await?;
        info!("mongodb schema ready");
        Ok(store)
    }

    fn users(&self) -> Collection<Document> {
        self.database.collection("users")
    }

    fn requests(&self) -> Collection<Document> {
        self.database.collection("mint_requests")
    }

    fn quotas(&self) -> Collection<Document> {
        self.database.collection("quotas")
    }

    fn failures(&self) -> Collection<Document> {
        self.database.collection("mint_failures")
    }

    async fn ensure_indexes(&self) -> Result<()> {
        let unique = IndexOptions::builder().unique(true).build();
        self.users()
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"channel": 1, "handle": 1})
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;

        self.requests()
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"status": 1, "requested_at": 1})
                    .options(IndexOptions::builder().build())
                    .build(),
                None,
            )
            .await?;

        self.quotas()
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"user_id": 1, "day": 1})
                    .options(unique)
                    .build(),
                None,
            )
            .await?;

        Ok(())
    }

    fn user_doc(user: &User) -> Document {
        doc! {
            "id": user.id.to_string(),
            "channel": user.channel.as_str(),
            "handle": &user.handle,
            "role": user.role.as_str(),
            "domain": user.domain.clone().map(Bson::String).unwrap_or(Bson::Null),
            "last_seen_at": Bson::DateTime(mongodb::bson::DateTime::from_chrono(user.last_seen_at)),
        }
    }

    fn request_doc(request: &MintRequest) -> Document {
        doc! {
            "id": request.id.to_string(),
            "user_id": request.user_id.to_string(),
            "channel": request.channel.as_str(),
            "amount": request.amount as i64,
            "status": request.status.as_str(),
            "tx_hash": request.tx_hash.clone().map(Bson::String).unwrap_or(Bson::Null),
            "error": request.error.clone().map(Bson::String).unwrap_or(Bson::Null),
            "requested_at": Bson::DateTime(mongodb::bson::DateTime::from_chrono(request.requested_at)),
            "processed_at": request
                .processed_at
                .map(|dt| Bson::DateTime(mongodb::bson::DateTime::from_chrono(dt)))
                .unwrap_or(Bson::Null),
            "attempt": request.attempt as i64,
        }
    }

    fn quota_doc(user_id: Uuid, day: NaiveDate) -> Document {
        doc! {
            "id": Uuid::new_v4().to_string(),
            "user_id": user_id.to_string(),
            "day": day.to_string(),
            "minted_total": 0i64,
            "success_count": 0i64,
        }
    }

    fn doc_to_user(doc: Document) -> Result<User> {
        Ok(User {
            id: Uuid::parse_str(doc.get_str("id")?)?,
            channel: channel_from_db(doc.get_str("channel")?)?,
            handle: doc.get_str("handle")?.to_string(),
            role: role_from_db(doc.get_str("role")?)?,
            domain: match doc.get("domain") {
                Some(Bson::String(value)) => Some(value.clone()),
                _ => None,
            },
            last_seen_at: doc.get_datetime("last_seen_at")?.to_chrono(),
        })
    }

    fn doc_to_request(doc: Document) -> Result<MintRequest> {
        Ok(MintRequest {
            id: Uuid::parse_str(doc.get_str("id")?)?,
            user_id: Uuid::parse_str(doc.get_str("user_id")?)?,
            channel: channel_from_db(doc.get_str("channel")?)?,
            amount: doc.get_i64("amount")? as u64,
            status: status_from_db(doc.get_str("status")?)?,
            tx_hash: match doc.get("tx_hash") {
                Some(Bson::String(value)) => Some(value.clone()),
                _ => None,
            },
            error: match doc.get("error") {
                Some(Bson::String(value)) => Some(value.clone()),
                _ => None,
            },
            requested_at: doc.get_datetime("requested_at")?.to_chrono(),
            processed_at: match doc.get("processed_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono()),
                _ => None,
            },
            attempt: doc.get_i64("attempt")? as u16,
        })
    }

    fn doc_to_quota(doc: Document) -> Result<Quota> {
        Ok(Quota {
            id: Uuid::parse_str(doc.get_str("id")?)?,
            user_id: Uuid::parse_str(doc.get_str("user_id")?)?,
            day: NaiveDate::parse_from_str(doc.get_str("day")?, "%Y-%m-%d")?,
            minted_total: doc.get_i64("minted_total")? as u64,
            success_count: doc.get_i64("success_count")? as u64,
        })
    }
}

#[async_trait::async_trait]
impl UserRepository for MongoStore {
    async fn upsert_user(&self, user: &User) -> Result<()> {
        let filter = doc! {"id": user.id.to_string()};
        let update = doc! {"$set": Self::user_doc(user)};
        let options = UpdateOptions::builder().upsert(true).build();
        self.users().update_one(filter, update, options).await?;
        Ok(())
    }

    async fn find_user(&self, channel: &str, handle: &str) -> Result<Option<User>> {
        let filter = doc! {"channel": channel, "handle": handle};
        let result = self.users().find_one(filter, None).await?;
        result.map(Self::doc_to_user).transpose()
    }

    async fn set_role(&self, user_id: Uuid, role: Role) -> Result<()> {
        let filter = doc! {"id": user_id.to_string()};
        let update = doc! {"$set": {"role": role.as_str()}};
        self.users().update_one(filter, update, None).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MintRepository for MongoStore {
    async fn enqueue(&self, request: &MintRequest) -> Result<()> {
        let doc = Self::request_doc(request);
        self.requests()
            .update_one(
                doc! {"id": request.id.to_string()},
                doc! {"$set": doc},
                UpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }

    async fn next_pending(&self) -> Result<Option<MintRequest>> {
        let now = mongodb::bson::DateTime::from_chrono(Utc::now());
        let update = doc! {
            "$set": {"status": MintStatus::Processing.as_str(), "processed_at": now},
            "$inc": {"attempt": 1},
        };
        let options = FindOneAndUpdateOptions::builder()
            .sort(doc! {"requested_at": 1})
            .return_document(ReturnDocument::After)
            .build();
        let doc = self
            .requests()
            .find_one_and_update(
                doc! {"status": MintStatus::Pending.as_str()},
                update,
                options,
            )
            .await?;

        doc.map(Self::doc_to_request).transpose()
    }

    async fn update_status(&self, request_id: Uuid, status: MintStatus) -> Result<()> {
        let processed_at = match status {
            MintStatus::Completed | MintStatus::Failed => {
                Some(mongodb::bson::DateTime::from_chrono(Utc::now()))
            }
            _ => None,
        };
        let mut set_doc = doc! {"status": status.as_str()};
        if let Some(ts) = processed_at {
            set_doc.insert("processed_at", ts);
        }
        self.requests()
            .update_one(
                doc! {"id": request_id.to_string()},
                doc! {"$set": set_doc},
                None,
            )
            .await?;
        Ok(())
    }

    async fn record_outcome(&self, outcome: &MintOutcome) -> Result<()> {
        let mut set_doc = doc! {
            "status": outcome.request.status.as_str(),
            "tx_hash": outcome.tx_hash.clone().map(Bson::String).unwrap_or(Bson::Null),
            "error": outcome.request.error.clone().map(Bson::String).unwrap_or(Bson::Null),
            "processed_at": outcome
                .request
                .processed_at
                .map(|dt| Bson::DateTime(mongodb::bson::DateTime::from_chrono(dt)))
                .unwrap_or(Bson::Null),
            "attempt": outcome.request.attempt as i64,
        };
        if set_doc.get("processed_at") == Some(&Bson::Null) {
            set_doc.insert(
                "processed_at",
                Bson::DateTime(mongodb::bson::DateTime::from_chrono(Utc::now())),
            );
        }

        self.requests()
            .update_one(
                doc! {"id": outcome.request.id.to_string()},
                doc! {"$set": set_doc},
                None,
            )
            .await?;

        if outcome.request.status == MintStatus::Completed {
            self.quotas()
                .update_one(
                    doc! {"user_id": outcome.request.user_id.to_string(), "day": outcome.request.requested_at.date_naive().to_string()},
                    doc! {"$inc": {"success_count": 1}},
                    None,
                )
                .await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl QuotaRepository for MongoStore {
    async fn record_mint(&self, user_id: Uuid, day: NaiveDate, amount: u64) -> Result<()> {
        self.quotas()
            .update_one(
                doc! {"user_id": user_id.to_string(), "day": day.to_string()},
                doc! {
                    "$setOnInsert": Self::quota_doc(user_id, day),
                    "$inc": {"minted_total": amount as i64}
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }

    async fn fetch_quota(&self, user_id: Uuid, day: NaiveDate) -> Result<Option<Quota>> {
        let doc = self
            .quotas()
            .find_one(
                doc! {"user_id": user_id.to_string(), "day": day.to_string()},
                None,
            )
            .await?;
        doc.map(Self::doc_to_quota).transpose()
    }
}

#[async_trait::async_trait]
impl ReportingRepository for MongoStore {
    async fn daily_summary(&self, day: NaiveDate) -> Result<Vec<DailyReportRow>> {
        let start = mongodb::bson::DateTime::from_chrono(DateTime::<Utc>::from_utc(
            day.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));
        let end_date = day + Duration::days(1);
        let end = mongodb::bson::DateTime::from_chrono(DateTime::<Utc>::from_utc(
            end_date.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));

        let pipeline = vec![
            doc! {
                "$match": {
                    "requested_at": {"$gte": start, "$lt": end}
                }
            },
            doc! {
                "$group": {
                    "_id": "$channel",
                    "total_amount": {"$sum": "$amount"},
                    "success_count": {
                        "$sum": {
                            "$cond": [{"$eq": ["$status", MintStatus::Completed.as_str()]}, 1, 0]
                        }
                    },
                    "failure_count": {
                        "$sum": {
                            "$cond": [{"$eq": ["$status", MintStatus::Failed.as_str()]}, 1, 0]
                        }
                    }
                }
            },
        ];

        let mut cursor = self.requests().aggregate(pipeline, None).await?;
        let mut rows = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            let channel = doc.get_str("_id").unwrap_or("unknown").to_string();
            let total_amount = doc.get_i64("total_amount").unwrap_or(0) as u64;
            let success_count = doc.get_i64("success_count").unwrap_or(0) as u64;
            let failure_count = doc.get_i64("failure_count").unwrap_or(0) as u64;
            rows.push(DailyReportRow {
                channel,
                total_amount,
                success_count,
                failure_count,
            });
        }
        Ok(rows)
    }

    async fn log_failure(&self, request_id: Uuid, when: DateTime<Utc>, reason: &str) -> Result<()> {
        self.failures()
            .insert_one(
                doc! {
                    "id": Uuid::new_v4().to_string(),
                    "request_id": request_id.to_string(),
                    "failed_at": mongodb::bson::DateTime::from_chrono(when),
                    "reason": reason
                },
                None,
            )
            .await?;
        Ok(())
    }
}
