mod convert;
mod data;
mod diagnostics;
mod error;
mod path;
mod queries;
mod resource;
mod stream;

use std::{collections::HashMap, sync::Arc};

use grafana_plugin_sdk::backend;
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio_postgres::{Client, Config, NoTls};

use convert::rows_to_frame;
use error::{Error, Result};

pub type SqlQueries = Arc<RwLock<HashMap<path::QueryId, queries::SelectStatement>>>;

#[derive(Clone, Debug, Default)]
pub struct MaterializePlugin {
    sql_queries: SqlQueries,
}

impl MaterializePlugin {
    async fn get_client(
        &self,
        datasource_settings: &backend::DataSourceInstanceSettings,
    ) -> Result<Client> {
        let settings: MaterializeDatasourceSettings =
            serde_json::from_value(datasource_settings.json_data.clone())
                .map_err(Error::InvalidDatasourceSettings)?;
        let (client, connection) = Config::new()
            .user(&settings.username)
            .host(&settings.host)
            .port(settings.port)
            .connect(NoTls)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(client)
    }
}

#[derive(Debug, Deserialize)]
struct MaterializeDatasourceSettings {
    host: String,
    port: u16,
    username: String,
}
