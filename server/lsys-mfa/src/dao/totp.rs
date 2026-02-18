use lsys_core::db::{query_string_field_max, Insert, SqlQuote, SqlSuffix, TableMeta, Update};
use lsys_core::{
    now_time, sql_format, valid_key, AppCore, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use sqlx::{MySql, Pool, Row};
use std::collections::HashSet;

use crate::model::{MfaStatus, MfaTotpModel};

use super::totp_util::{decode_base32, totp_code};
use super::{MfaError, MfaResult, MfaSubject};

#[derive(Clone, Copy)]
pub struct MfaTotpConfig {
    pub step_seconds: u64,
    pub digits: u32,
    pub window: i64,
}

impl Default for MfaTotpConfig {
    fn default() -> Self {
        Self {
            step_seconds: 30,
            digits: 6,
            window: 1,
        }
    }
}

impl MfaTotpConfig {
    /// Load configuration from `app.toml` via `AppCore`.
    ///
    /// Supported keys (both forms accepted):
    /// - `mfa_totp_step_seconds` or `mfa-totp-step-seconds`
    /// - `mfa_totp_digits` or `mfa-totp-digits`
    /// - `mfa_totp_window` or `mfa-totp-window`
    pub fn load(app_core: &AppCore) -> Self {
        let cfg = app_core.config.find(None);

        let mut out = Self::default();

        let step = cfg
            .get_int("mfa_totp_step_seconds")
            .ok()
            .or_else(|| cfg.get_int("mfa-totp-step-seconds").ok())
            .unwrap_or(out.step_seconds as i64);
        let digits = cfg
            .get_int("mfa_totp_digits")
            .ok()
            .or_else(|| cfg.get_int("mfa-totp-digits").ok())
            .unwrap_or(out.digits as i64);
        let window = cfg
            .get_int("mfa_totp_window")
            .ok()
            .or_else(|| cfg.get_int("mfa-totp-window").ok())
            .unwrap_or(out.window);

        // Clamp to safe/expected ranges.
        out.step_seconds = step.clamp(5, 300) as u64;
        out.digits = digits.clamp(4, 8) as u32;
        out.window = window.clamp(0, 5);

        out
    }
}

pub struct MfaTotpDao {
    db: Pool<MySql>,
    cfg: MfaTotpConfig,
}

impl MfaTotpDao {
    pub fn new(db: Pool<MySql>, cfg: Option<MfaTotpConfig>) -> Self {
        let mut config = cfg.unwrap_or_default();
        if config.step_seconds == 0 {
            config.step_seconds = 30;
        }
        Self { db, cfg: config }
    }

    pub async fn get_active(&self, subject: &MfaSubject) -> MfaResult<Option<MfaTotpModel>> {
        let data = sqlx::query_as::<_, MfaTotpModel>(&sql_format!(
            "select * from {} where app_id={} and user_data={} and status={} order by id desc limit 1",
            MfaTotpModel::table_name(),
            subject.app_id,
            subject.user_data,
            MfaStatus::Enable as i8,
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(data)
    }

    pub async fn is_enabled(&self, subject: &MfaSubject) -> MfaResult<bool> {
        Ok(self.get_active(subject).await?.is_some())
    }

    /// Batch check whether TOTP MFA is enabled for each subject.
    ///
    /// Returns a boolean vector aligned with `subjects`.
    pub async fn is_enabled_batch(&self, subjects: &[MfaSubject]) -> MfaResult<Vec<bool>> {
        if subjects.is_empty() {
            return Ok(Vec::new());
        }

        let user_data_max =
            query_string_field_max::<MfaTotpModel>(&self.db, &MfaTotpModel::USER_DATA)
                .await
                .len_or(32);

        // Validate all subjects
        for subject in subjects.iter() {
            let mut valid_param = ValidParam::default();
            valid_param.add(
                valid_key!("totp_user_data"),
                &subject.user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, user_data_max)),
            );
            valid_param.check()?;
        }

        // Query all enabled subjects in one go.
        // MySQL supports row constructor IN: (app_id, user_data) IN ((?, ?), ...)
        let mut qb = sqlx::QueryBuilder::<MySql>::new("select app_id, user_data from ");
        qb.push(MfaTotpModel::table_name());
        qb.push(" where status=").push_bind(MfaStatus::Enable as i8);
        qb.push(" and (app_id, user_data) in (");

        {
            let mut separated = qb.separated(",");
            for subject in subjects {
                separated
                    .push("(")
                    .push_bind(subject.app_id)
                    .push(",")
                    .push_bind(&subject.user_data)
                    .push(")");
            }
        }
        qb.push(")");

        let rows = qb.build().fetch_all(&self.db).await?;
        let mut enabled: HashSet<(u64, String)> = HashSet::with_capacity(rows.len());
        for row in rows {
            let app_id: u64 = row.try_get("app_id")?;
            let user_data: String = row.try_get("user_data")?;
            enabled.insert((app_id, user_data));
        }

        Ok(subjects
            .iter()
            .map(|s| enabled.contains(&(s.app_id, s.user_data.clone())))
            .collect())
    }

    /// Enable a new TOTP secret.
    ///
    /// Keeps history by inserting a new row and disabling older enabled rows.
    pub async fn enable_new_secret(
        &self,
        subject: &MfaSubject,
        secret_data: &str,
    ) -> MfaResult<u64> {
        let secret_data_max =
            query_string_field_max::<MfaTotpModel>(&self.db, &MfaTotpModel::SECRET_DATA)
                .await
                .len_or(128);
        let user_data_max =
            query_string_field_max::<MfaTotpModel>(&self.db, &MfaTotpModel::USER_DATA)
                .await
                .len_or(32);

        // Validate parameters
        let mut valid_param = ValidParam::default();
        valid_param
            .add(
                valid_key!("totp_secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(10, secret_data_max)),
            )
            .add(
                valid_key!("totp_user_data"),
                &subject.user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, user_data_max)),
            );
        valid_param.check()?;

        let time = now_time().unwrap_or_default();
        let status_disable = MfaStatus::Disable as i8;
        let status_enable = MfaStatus::Enable as i8;

        // Insert first (newest row wins), then best-effort disable older enabled rows.
        // This avoids transaction isolation issues where concurrent transactions cannot see
        // each other's uncommitted inserts.
        let res = Insert::<MfaTotpModel>::new()
            .set(MfaTotpModel::APP_ID, subject.app_id)
            .set(MfaTotpModel::USER_DATA, &subject.user_data)
            .set(MfaTotpModel::STATUS, status_enable)
            .set(MfaTotpModel::SECRET_DATA, secret_data)
            .set(MfaTotpModel::LAST_USED_STEP, 0u64)
            .set(MfaTotpModel::LAST_USED_TIME, 0u64)
            .set(MfaTotpModel::ADD_TIME, time)
            .set(MfaTotpModel::CHANGE_TIME, time)
            .execute(&self.db)
            .await?;
        let new_id = res.last_insert_id();

        Update::<MfaTotpModel>::new()
            .set(MfaTotpModel::STATUS, status_disable)
            .set(MfaTotpModel::CHANGE_TIME, time)
            .execute(
                SqlSuffix::Where(&sql_format!(
                    "app_id={} and user_data={} and status={} and id<{}",
                    subject.app_id,
                    subject.user_data,
                    MfaStatus::Enable as i8,
                    new_id,
                )),
                &self.db,
            )
            .await?;

        Ok(new_id)
    }

    pub async fn disable(&self, subject: &MfaSubject) -> MfaResult<()> {
        let time = now_time().unwrap_or_default();
        let status_disable = MfaStatus::Disable as i8;
        Update::<MfaTotpModel>::new()
            .set(MfaTotpModel::STATUS, status_disable)
            .set(MfaTotpModel::CHANGE_TIME, time)
            .execute(
                SqlSuffix::Where(&sql_format!(
                    "app_id={} and user_data={} and status={}",
                    subject.app_id,
                    subject.user_data,
                    MfaStatus::Enable as i8,
                )),
                &self.db,
            )
            .await?;
        Ok(())
    }

    pub async fn verify_totp(&self, subject: &MfaSubject, code: &str) -> MfaResult<()> {
        let user_data_max =
            query_string_field_max::<MfaTotpModel>(&self.db, &MfaTotpModel::USER_DATA)
                .await
                .len_or(32);

        // Validate parameters
        let mut valid_param = ValidParam::default();
        let code_trimmed = code.trim();
        valid_param
            .add(
                valid_key!("totp_code"),
                &code_trimmed,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Numeric)
                    .add_rule(ValidStrlen::eq(self.cfg.digits as u64)),
            )
            .add(
                valid_key!("totp_user_data"),
                &subject.user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, user_data_max)),
            );
        valid_param.check()?;

        let row = self
            .get_active(subject)
            .await?
            .ok_or(MfaError::NotEnabled)?;
        let now = now_time().unwrap_or_default();
        let now_step = now / self.cfg.step_seconds;

        let secret = decode_base32(&row.secret_data)?;
        let code = code_trimmed;

        let mut matched_step: Option<u64> = None;
        for offset in -self.cfg.window..=self.cfg.window {
            let step = if offset.is_negative() {
                now_step.saturating_sub(offset.unsigned_abs())
            } else {
                now_step.saturating_add(offset as u64)
            };
            let gen = totp_code(&secret, step, self.cfg.digits)?;
            if gen == code {
                matched_step = Some(step);
                break;
            }
        }

        let used_step = matched_step.ok_or(MfaError::VerifyFailed)?;
        if used_step <= row.last_used_step {
            return Err(MfaError::Replay);
        }

        let time = now;
        Update::<MfaTotpModel>::new()
            .set(MfaTotpModel::LAST_USED_STEP, used_step)
            .set(MfaTotpModel::LAST_USED_TIME, time)
            .set(MfaTotpModel::CHANGE_TIME, time)
            .execute(SqlSuffix::Where(&sql_format!("id={}", row.id)), &self.db)
            .await?;

        Ok(())
    }
}
