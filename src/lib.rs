mod config;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use anyhow::{Context, Result, anyhow};
use kovi::{
    Message, PluginBuilder as plugin, RuntimeBot,
    bot::{message::Segment, runtimebot::kovi_api::SetAccessControlList},
    event::GroupMsgEvent,
    log::{error, info},
    serde_json::json,
    tokio::{
        fs::{self, File},
        io::AsyncWriteExt,
        sync::RwLock,
    },
};
use reqwest::Client;
use tinyrand::{Rand, StdRand, Wyrand};

const PLUGIN_NAME: &str = "kovi-plugin-picture-store";
static RAND: OnceLock<RwLock<Wyrand>> = OnceLock::new();

#[kovi::plugin]
async fn main() {
    let _ = RAND.set(RwLock::new(StdRand::default()));

    let bot = plugin::get_runtime_bot();
    let client = Arc::new(reqwest::ClientBuilder::new().build().unwrap());

    let config = config::init(&bot).await.unwrap();

    if let Some(groups) = &config.allow_groups {
        bot.set_plugin_access_control(PLUGIN_NAME, true).unwrap();
        bot.set_plugin_access_control_list(
            PLUGIN_NAME,
            true,
            SetAccessControlList::Adds(groups.clone()),
        )
        .unwrap();
    } else {
        bot.set_plugin_access_control(PLUGIN_NAME, false).unwrap();
    }

    plugin::on_group_msg({
        let bot = bot.clone();
        let client = client.clone();
        move |msg| on_group_msg(msg, bot.clone(), client.clone())
    });

    info!("[pic-store] Ready to be a cyber hamster!");
}

async fn on_group_msg(event: Arc<GroupMsgEvent>, bot: Arc<RuntimeBot>, client: Arc<Client>) {
    if let Err(err) = handle(event.clone(), bot.clone(), client).await {
        error!("[{PLUGIN_NAME}] Error when handling message: \n{err}");

        let cfg = config::CONFIG.get().unwrap();
        bot.send_group_msg(event.group_id, cfg.err_msg.clone());
    }
}

async fn handle(
    event: Arc<GroupMsgEvent>,
    bot: Arc<RuntimeBot>,
    client: Arc<Client>,
) -> Result<()> {
    let cfg = config::CONFIG.get().unwrap();

    let mut is_at = false;
    let mut reply_id = None;
    let mut is_send = false;
    let mut is_store = false;
    let mut target = None;

    for msg in event.message.clone() {
        match msg.type_.as_str() {
            "at" => {
                let who = msg
                    .data
                    .get("qq")
                    .ok_or(anyhow!("Unable to find 'qq' field in 'at' segment data"))?
                    .as_str()
                    .ok_or(anyhow!(""))?;
                let who = i64::from_str_radix(who, 10)?;
                is_at |= who == cfg.self_id;
            }
            "reply" => {
                reply_id = msg
                    .data
                    .get("id")
                    .ok_or(anyhow!("Unable to find 'id' field in 'reply' segment data",))?
                    .as_str()
                    .map_or(None, |i| Some(i.to_string()))
            }
            "text" => {
                let content = msg
                    .data
                    .get("text")
                    .ok_or(anyhow!(
                        "Unable to find 'text' field in 'text' segment data",
                    ))?
                    .as_str()
                    .ok_or(anyhow!("Unable to parse text data from 'text' field"))?
                    .trim();
                if content.starts_with(cfg.send_trigger.as_str()) {
                    let arg = content[cfg.send_trigger.len()..].to_string();
                    target = if arg.len() > 0 { Some(arg) } else { None };
                    is_send = true
                }
                if content.starts_with(cfg.store_trigger.as_str()) {
                    let arg = content[cfg.store_trigger.len()..].to_string();
                    target = if arg.len() > 0 { Some(arg) } else { None };
                    is_store = true
                }
            }
            _ => {}
        }
    }

    if !is_at {
        info!("[{PLUGIN_NAME}] Not mentioned, don't care");
        return Ok(());
    }

    if is_store
        && let Some(admins) = cfg.allow_admins.clone()
        && !admins.contains(&event.user_id)
    {
        info!(
            "[{PLUGIN_NAME}] User {} is not allowed to store",
            event.user_id
        );
        bot.send_group_msg(event.group_id, cfg.no_perm_msg.clone());
        return Ok(());
    }

    if is_send && is_store {
        info!("[{PLUGIN_NAME}] Send & Store in one message, send confused reply");
        bot.send_group_msg(event.group_id, cfg.cmd_err_msg.clone());
        return Ok(());
    }

    if is_store && reply_id.is_none() {
        info!("[{PLUGIN_NAME}] Store without reply");
        bot.send_group_msg(event.group_id, cfg.no_source_msg.clone());
        return Ok(());
    }

    if (is_send || is_store) && target.is_none() {
        info!("[{PLUGIN_NAME}] Send / Store with no target provided");
        bot.send_group_msg(event.group_id, cfg.no_target_msg.clone());
        return Ok(());
    }

    let target = target.clone().unwrap().trim().to_string();
    let target = if target.len() > 128 {
        target[..128].to_string()
    } else {
        target
    };

    let folder = target
        .as_bytes()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join("");

    info!("[{PLUGIN_NAME}] Query '{target}' -> '{folder}'");
    let folder = bot.get_data_path().join("pictures").join(&folder);

    if is_send {
        if !folder.exists() {
            info!("[{PLUGIN_NAME}] Directory does not exist, sending fallback message");
            bot.send_group_msg(event.group_id, cfg.no_pic_msg.clone());
            return Ok(());
        }

        let mut results = fs::read_dir(&folder).await.context(format!(
            "Unable to read directory {}",
            folder.to_string_lossy()
        ))?;

        let mut files: Vec<PathBuf> = vec![];

        info!("[{PLUGIN_NAME}] Collecting available pictures for '{target}'");
        loop {
            let entry = results.next_entry().await.context(format!(
                "Unable to read directory '{}'",
                folder.to_string_lossy()
            ))?;

            if let None = entry {
                break;
            }

            let entry = entry.unwrap();
            files.push(entry.path());
        }

        if files.is_empty() {
            info!("[{PLUGIN_NAME}] No picture found, sending fallback message");
            bot.send_group_msg(event.group_id, cfg.no_pic_msg.clone());
            return Ok(());
        }

        let idx = RAND.get().unwrap().write().await.next_usize() % files.len();
        let file = &files[idx];

        bot.send_group_msg(
            event.group_id,
            Message::from_value(json!(
                [
                    {
                        "type": "image",
                        "data": {
                            "file": file
                        }
                    }
                ]
            ))
            .context("Error when constructing message")?,
        );
        info!("[{PLUGIN_NAME}] Picture '{}' sent", file.to_string_lossy());
    } else if is_store {
        if !folder.exists() {
            fs::create_dir_all(&folder).await?;
        }

        let reply_id = reply_id.clone().unwrap();
        let reply_id = i32::from_str_radix(&reply_id.as_str(), 10)
            .context(format!("Unable to convert message id '{reply_id}' to i32"))?;

        let source = bot
            .get_msg(reply_id)
            .await
            .map_err(|_| anyhow!("Failed to get message from onebot response"))?;
        let source = Message::from_value(
            source
                .data
                .get("message")
                .ok_or(anyhow!("Unable to get 'message' from onebot response"))?
                .clone(),
        )
        .context("Unable to parse valid message object from onebot response")?;
        if !source.contains("image") {
            info!("[{PLUGIN_NAME}] Store without images in reply");
            bot.send_group_msg(event.group_id, cfg.no_source_msg.clone());
            return Ok(());
        }

        for seg in source {
            if seg.type_ != "image" {
                continue;
            }

            if let Err(e) = handle_pictures(seg, &folder, client.clone()).await {
                error!("[{PLUGIN_NAME}] Error when downloading picture: {e}, Skipping");
            }
        }

        bot.send_group_msg(event.group_id, cfg.success_msg.clone());
    }

    Ok(())
}

async fn handle_pictures(seg: Segment, folder: &PathBuf, client: Arc<Client>) -> Result<()> {
    let url = seg
        .data
        .get("url")
        .ok_or(anyhow!("Unable to get 'url' in 'image' segement data"))?
        .as_str()
        .ok_or(anyhow!("Unable to parse 'url' as str"))?
        .to_string();

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("Unable to download picture '{url}', skipping"));
    }

    let bytes = resp.bytes().await.context("Failed to read response body")?;
    let typ = infer::get(&bytes).context("Could not determine file type")?;
    let checksum = md5::compute(&bytes);

    let name = format!("{:x}.{}", checksum, typ.extension());
    let path = folder.join(Path::new(&name));
    let mut file = File::create(&path).await.context(format!(
        "Failed to create output file {}",
        path.to_string_lossy()
    ))?;
    file.write_all(&bytes)
        .await
        .context("Failed to write file")?;

    info!("Picture '{name}' saved");

    Ok(())
}
