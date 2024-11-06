use log::{debug, warn};
use std::{path::PathBuf, sync::Arc};
use tokio::task::JoinHandle;

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{middleware::FetchClient, r2client::R2Client};

use super::utils::{get_assets_url, get_cdragon_url, AssetsTypeTrait, Config, FALLBACK_CONFIG};

pub trait CollecTasks {
    fn collect_tasks(&self, config: Arc<Config>) -> Vec<JoinHandle<Result<()>>>;
}

pub trait ToTask {
    fn to_task(&self, config: Arc<Config>) -> Option<JoinHandle<Result<()>>>;

    fn to_edge_task(url: &str, config: Arc<Config>) -> Result<JoinHandle<Result<()>>> {
        let url = url.trim().to_string();
        if url.is_empty() || !url.starts_with("/lol-game-data/assets/") {
            warn!("Invalid url: {}", url);
            return Err(anyhow!("Invalid url: {}", url));
        }
        let name = url.clone().trim_start_matches('/').to_string();
        let download_url = get_cdragon_url(&url, &config);
        let fallback_url = get_cdragon_url(&url, &FALLBACK_CONFIG);

        let r2 = R2Client::try_from_env()?;
        let handle = tokio::spawn(async move {
            match r2.upload_file(&download_url, &name).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("Failed to upload: {}, try fallback", e);
                    r2.upload_file(&fallback_url, &name).await
                }
            }
        });

        Ok(handle)
    }

    fn to_download_task(url: &str, config: Arc<Config>) -> Result<JoinHandle<Result<()>>> {
        let url = url.trim();
        if url.is_empty() || !url.starts_with("/lol-game-data/assets/") {
            warn!("Invalid url: {}", url);
            return Err(anyhow!("Invalid url: {}", url));
        }
        let save_path = config.base_path.join(url.trim_start_matches('/'));
        if save_path.exists() {
            debug!("File exists: {:?}", save_path);
            return Err(anyhow!("File exists: {:?}", save_path));
        }

        let url_clone = url.to_string();
        let handle = tokio::spawn(async move {
            FetchClient::default()
                .get_bytes(&url_clone, &config)
                .await?
                .save_file(&save_path)
                .await
        });
        Ok(handle)
    }
}

pub trait SaveFile {
    fn save_file(
        &self,
        save_path: &PathBuf,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl SaveFile for Vec<u8> {
    async fn save_file(&self, save_path: &PathBuf) -> Result<()> {
        let parent = save_path.parent().ok_or_else(|| anyhow!("Invalid path"))?;
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| anyhow!("Failed to create dir: {:?}, {}", parent, e))?;
        }
        tokio::fs::write(save_path, self)
            .await
            .map_err(|e| e.into())
            .and_then(|_| {
                debug!("Save file: {:?}", save_path);
                Ok(())
            })
    }
}

pub trait FromUrl: DeserializeOwned + AssetsTypeTrait {
    fn from_url(config: &Config) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        Self: Sync,
    {
        async {
            let assets_type = Self::assets_type(); // 使用 Self 调用关联函数
            let url = get_assets_url(&assets_type, &config.language, &config.version);
            FetchClient::default()
                .get(&url)
                .await?
                .json::<Self>() // 明确指定反序列化的目标类型
                .await
                .map_err(|e| e.into())
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    pub region: RegionEnum,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rarity {
    pub region: RegionEnum,
    pub rarity: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RarityEnum {
    #[serde(rename = "kEpic")]
    Epic,

    #[serde(rename = "kLegendary")]
    Legendary,

    #[serde(rename = "kMythic")]
    Mythic,

    #[default]
    #[serde(rename = "kNoRarity")]
    NoRarity,

    #[serde(rename = "kRare")]
    Rare,

    #[serde(rename = "kTranscendent")]
    Transcendent,

    #[serde(rename = "kUltimate")]
    Ultimate,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegionEnum {
    #[default]
    #[serde(rename = "")]
    Empty,

    #[serde(rename = "ID")]
    Id,

    #[serde(rename = "PH")]
    Ph,

    #[serde(rename = "tencent")]
    RegionTencent,

    #[serde(rename = "riot")]
    Riot,

    #[serde(rename = "SG")]
    Sg,

    #[serde(rename = "TENCENT")]
    Tencent,

    #[serde(rename = "TH")]
    Th,

    #[serde(rename = "TW")]
    Tw,

    #[serde(rename = "VN")]
    Vn,
}
