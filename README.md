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

Picture-Store 可以通过 `toml` 文件进行配置。如果配置文件不存在，则使用默认配置。  

```toml
# 自己的 ID
self_id = 123456789

# 在不同情况下 bot 反馈的信息
success_msg = "okk~"
err_msg = "好像出了点问题，看看日志？"
cmd_err_msg = "一边发一边收会坏的 (悲"
no_target_msg = "需要啥"
no_source_msg = "似乎没看到图"
no_pic_msg = "这是什么这是什么"
no_perm_msg = "达咩，不听你的"

# 允许在如下群组里工作
[[allow_groups]]
id = 11451419

# 并且可以限制群组使用的字段
[[allow_groups]]
id = 789456123
store = false      # 禁止保存
tokens = ["猫猫"]   # 只能发这一种图片



```  

配置文件应放置于编译后与可执行文件同级的 `data/kovi-plugin-picture-store/config.toml` 中。