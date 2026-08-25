use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::{Context, Result};
use kovi::{RuntimeBot, tokio::fs};
use serde::Deserialize;

use crate::consts::CONFIG_PATH;

pub(crate) static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Deserialize, Clone)]
pub(crate) struct GroupPerm {
    pub(crate) id: i64,
    pub(crate) send: Option<bool>,
    pub(crate) store: Option<bool>,
    pub(crate) tokens: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct ConfigFile {
    pub(crate) allow_groups: Option<Vec<GroupPerm>>,
    pub(crate) allow_admins: Option<Vec<i64>>,
    pub(crate) self_id: i64,

    pub(crate) send_trigger: Option<String>,
    pub(crate) store_trigger: Option<String>,
    pub(crate) success_msg: Option<String>,
    pub(crate) err_msg: Option<String>,
    pub(crate) cmd_err_msg: Option<String>,
    pub(crate) no_target_msg: Option<String>,
    pub(crate) no_source_msg: Option<String>,
    pub(crate) no_pic_msg: Option<String>,
    pub(crate) no_perm_msg: Option<String>,
}

pub(crate) struct Config {
    pub(crate) allow_groups: Option<HashMap<i64, GroupPerm>>,
    pub(crate) allow_admins: Option<Vec<i64>>,
    pub(crate) self_id: i64,

    pub(crate) send_trigger: String,
    pub(crate) store_trigger: String,
    pub(crate) success_msg: String,
    pub(crate) err_msg: String,
    pub(crate) cmd_err_msg: String,
    pub(crate) no_target_msg: String,
    pub(crate) no_source_msg: String,
    pub(crate) no_pic_msg: String,
    pub(crate) no_perm_msg: String,
}

pub(crate) async fn init(bot: &Arc<RuntimeBot>) -> Result<&Config> {
    let config_path = bot.get_data_path().join(CONFIG_PATH);
    let config_txt = fs::read_to_string(&config_path)
        .await
        .with_context(|| format!("Failed to read config file at {}", config_path.display()))?;
    let config: ConfigFile = toml::from_str(&config_txt)?;

    let map = config.allow_groups.map_or(None, |e| {
        Some(
            e.iter()
                .map(|p| (p.id, p.clone()))
                .collect::<HashMap<_, _>>(),
        )
    });

    Ok(CONFIG.get_or_init(|| Config {
        allow_groups: map,
        allow_admins: config.allow_admins,
        self_id: config.self_id,
        send_trigger: config.send_trigger.unwrap_or("发".to_string()),
        store_trigger: config.store_trigger.unwrap_or("这是".to_string()),
        success_msg: config.success_msg.unwrap_or("图片已记录".to_string()),
        err_msg: config.err_msg.unwrap_or("出现神秘问题".to_string()),
        cmd_err_msg: config
            .cmd_err_msg
            .unwrap_or("无法同时发送和记录图片".to_string()),
        no_target_msg: config
            .no_target_msg
            .unwrap_or("请指定要收发的内容".to_string()),
        no_source_msg: config.no_source_msg.unwrap_or("请引用一张图片".to_string()),
        no_pic_msg: config.no_pic_msg.unwrap_or("没有找到此类图片".to_string()),
        no_perm_msg: config.no_perm_msg.unwrap_or("权限不足".to_string()),
    }))
}
