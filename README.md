# Picture-Store

Picture-Store 是基于 [Kovi](https://kovi.thricecola.com/) 框架的自动图像存储/发送插件，可以按关键词收藏图片并随机发送。

用户可以引用图片消息并 @ bot，发送 "这是xx" 来收藏图片；后续可以 @ bot 发送 "发xx" 来要求 bot 发送随机图片。

## 安装

1. 根据[教程](https://kovi.thricecola.com/start/fast.html)创建一个 Kovi 工程
2. 在项目根目录运行
```bash
cargo add kovi-plugin-picture-store
```

3. 在 `build_bot!` 宏中传入插件
```rust
let bot = build_bot!(kovi-plugin-picture-store /* 和其他你正在使用的插件，用 , 分割 */ );
```

## 配置

Copycat 可以通过 `toml` 文件进行配置。如果配置文件不存在，则使用默认配置。  

```toml
# 自己的 ID, 用于判断是否在 at 自己
self_id = 12345678

# 允许收/发的群组，若没有这一项，则所有群组都可收发图片
allow_groups = [87654321]

# 允许收藏图片的用户，若没有这一项，则所有用户均可以让 bot 收藏图片
allow_admins = [23456789]

# 以下均为可选配置，可以自定义 bot 的发言风格
send_trigger = "发送图片命令前缀，默认值为'发'"
store_trigger = "收藏图片命令前缀，默认值为'这是'"

success_msg = "图片收藏成功时的回复"
err_msg = "出现未知错误时的回复"
cmd_err_msg = "用户同时要求收藏 & 发图片时的回复"
no_target_msg = "用户没有提供要发送/收藏的类型名称时的回复"
no_source_msg = "用户要求收藏，但没有提供图片引用时的回复"
no_pic_msg = "用户提供的名称不能存在时的回复"
no_perm_msg = "用户不在 allow_admins 时要求收藏图片的回复"

```  

配置文件应放置于编译后与可执行文件同级的 `data/kovi-plugin-picture-store/config.toml` 中。