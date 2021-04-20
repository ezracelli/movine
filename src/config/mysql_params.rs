use crate::errors::{Error, Result};
use serde::Deserialize;
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MysqlParams {
    pub user: String,
    pub password: Option<String>,
    pub host: String,
    pub database: String,
    pub port: i32,
    pub sslcert: Option<PathBuf>,
}

impl TryFrom<&[&RawMysqlParams]> for MysqlParams {
    type Error = Error;

    fn try_from(value: &[&RawMysqlParams]) -> Result<MysqlParams> {
        let params = value.iter().fold(RawMysqlParams::default(), |mut acc, x| {
            acc.user = x.user.to_owned().or(acc.user);
            acc.password = x.password.to_owned().or(acc.password);
            acc.host = x.host.to_owned().or(acc.host);
            acc.database = x.database.to_owned().or(acc.database);
            acc.port = x.port.to_owned().or(acc.port);
            acc.sslcert = x.sslcert.to_owned().or(acc.sslcert);
            acc
        });

        match params {
            RawMysqlParams {
                user: Some(user),
                password,
                database: Some(database),
                host: Some(host),
                port: Some(port),
                sslcert,
            } => Ok(Self {
                user,
                password,
                host,
                database,
                port,
                sslcert,
            }),
            p => Err(Error::MysqlParamError {
                user: p.user.is_some(),
                password: p.password.is_some(),
                database: p.database.is_some(),
                host: p.host.is_some(),
                port: p.port.is_some(),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawMysqlParams {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub database: Option<String>,
    pub port: Option<i32>,
    pub sslcert: Option<PathBuf>,
}

impl RawMysqlParams {
    pub fn load_from_env() -> Result<Self> {
        let params = envy::prefixed("MYSQL_").from_env()?;
        Ok(params)
    }

    pub fn is_any(&self) -> bool {
        self.user.is_some()
            || self.password.is_some()
            || self.host.is_some()
            || self.database.is_some()
            || self.port.is_some()
    }
}

impl Default for RawMysqlParams {
    fn default() -> Self {
        Self {
            user: None,
            password: None,
            host: None,
            database: None,
            port: Some(5432),
            sslcert: None,
        }
    }
}
