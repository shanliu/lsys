use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use image::{ImageBuffer, Rgb};
use lsys_core::cache::{LocalCache, LocalCacheClearItem};

use crate::model::BarcodeCreateModel;

use super::BarCodeDao;

pub enum BarCodeLocalCacheClear {
    CreateModel(Arc<LocalCache<u64, BarcodeCreateModel>>),
    #[allow(clippy::type_complexity)]
    CreateBuffer(Arc<LocalCache<String, ImageBuffer<Rgb<u8>, Vec<u8>>>>),
}

impl BarCodeLocalCacheClear {
    pub fn new_clears(bardao: &BarCodeDao) -> Vec<Self> {
        vec![
            Self::CreateModel(bardao.create_model.clone()),
            Self::CreateBuffer(bardao.create_render.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem<'_> for BarCodeLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::CreateModel(cache) => cache.config().cache_name,
            Self::CreateBuffer(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            Self::CreateModel(cache) => {
                let key = &u64::from_str(msg).map_err(|e| e.to_string())?;
                cache.del(key).await
            }
            Self::CreateBuffer(cache) => cache.del(&msg.to_owned()).await,
        };
        Ok(())
    }
}
