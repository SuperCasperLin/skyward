# 自有发行与在线更新

把在线更新从「读上游 `kirklin/skyward` 的 release」改成「读我们自己仓库的 release」，
让手机里的 Recovery 直接更新我们发布的镜像（官方 → 我们 → 手机）。

> **当前状态（2026-11-15）**：代码已改好，发行仓库为 **`SuperCasperLin/skyward`**（public fork，
> 代码推在 `nuwa` 分支），`release = "0.2.4"` 与移植版本号尾段一致。release 上线后在线更新即生效；
> 在 release 发布前设备点「检查更新」只会查到暂无 release，**不会回退到官方包**。

## 为什么要改

原来的 `skyward-net/src/github.rs` 把 `LATEST` 写死为
`https://api.github.com/repos/kirklin/skyward/releases/latest`，而运行中的版本用
`skyward_info::UPSTREAM`（=`CARGO_PKG_VERSION` = `0.2.0`）去比。上游现在最新是 `v0.3.1`，
于是手机会提示「有新版本」，且官方 asset
`skyward-v0.3.1-nuwa-OS3.0.312.0.WMBCNXM.zip` 的命名、SHA-256、`device.toml` 里的
codename 全都满足校验——**点一次更新就会把官方未实测的 nuwa 包刷进 recovery，
丢掉我们全部定制**。改造后这个回退路径不存在了。

## 版本序列

**在线更新的 tag 就是移植版本号本身**，界面上只有一个号。复合串
`SW-v0.2.0-nuwa-v0.2.4` 不是 semver（`SW-` 前缀、机型段），没法直接拿去比较，所以
`[update].release` 单独存尾段的纯 semver 形式，两者必须同步改：

| 用途 | 值 | 位置 |
|---|---|---|
| About 页 / 镜像名 | `SW-v0.2.0-nuwa-v0.2.4` | `VERSION`、`skyward-info::VERSION` |
| 在线更新比较 | `0.2.4` → tag `v0.2.4` | `[update].release` |

规则：nuwa 这条适配线的移植尾段**只增不重置**（0.2.4 → 0.2.5 → …，跨上游基线也继续
递增，不回到 v0.1.0），于是检查更新界面显示的号永远和 About 页一致。
tag 一旦发布不要改动或重建，否则已刷入的设备会比较出错乱。
生成器会用 `[update].release` 反推 tag 与更新包文件名，避免两者写歪。
（曾用过独立的 `v1.0.0` 序列，2026-11-16 起废弃：两套号让「当前 0.2.4、最新 1.0.0、
却是最新」这种界面自相矛盾，已全部对齐为移植版本号尾段。）

## 配置字段

`[update]` 段（**`devices/nuwa.toml` 与 `upstream/devices/nuwa.toml` 两份必须同步**——
`build.sh` 在 `upstream/` 目录里执行，打包读的是后者）：

```toml
[update]
server  = "https://update.miui.com/updates/miotaV3.php"   # 厂商系统 OTA 检查，不动
repo    = "SuperCasperLin/skyward"   # 留空 = 不联网、直接回答「已是最新」
release = "0.2.4"     # 纯 semver，与移植版本号尾段一致，tag 为 v0.2.4
```

`repo` 留空时 `updates.rs` 会记一行 kmsg 并返回 `newer: false`，**不会回退到上游官方**。
部署时只填这一个值、重新打包即可，不必再改代码。

## 两种 ZIP，别混用

| | 卡刷包（flashable ZIP） | 在线更新包（release ZIP） |
|---|---|---|
| 谁用 | 第三方 REC（TWRP / OrangeFox） | Skyward 自己的「在线更新」页 |
| 内容 | `update-binary` + `updater-script` + `recovery.img` + 说明 | **恰好一个 `.img` 条目** |
| 生成器 | `tools/make_flashable_zip.py` | `tools/make_release_zip.py` |
| 命名 | `<VERSION>-flashable.zip` | `skyward-<tag>-<codename>-<stock>.zip` |
| 文档 | [FLASHABLE.md](FLASHABLE.md) | 本文件 |

`skyward_install::upgrade::unzip()` 拒绝多条目压缩包，`check()` 还会读镜像里的
`skyward/device.toml` 比对 codename。`skyward_net::github::newest()` 要求 asset 以
`skyward-{tag}-{codename}-` 开头、以 `.zip` 结尾，且 GitHub API 给出
`digest: sha256:...`（上传 asset 时 GitHub 自动生成）。

`out/release/` 里的 `skyward-v0.2.4-nuwa-OS3.0.312.0.WMBCNXM.zip` 必须用**重新打包后的镜像**
再生成一次才是正式发布包；任何用旧镜像 `59e568a0…` 生成的包都只是格式验证产物，不要上传。

## 部署流程

1. 确认 `devices/nuwa.toml` 和 `upstream/devices/nuwa.toml` 的 `repo` 均为 `SuperCasperLin/skyward`（两份都要一致）。
2. `bash build.sh` 同版本重打镜像（哈希会变）。同步 `CHANGELOG.md` / `README.md` 的哈希，
   图内 `system/bin/recovery` 哈希要与本地 `skyward` 二进制一致。
3. **先在自己的 13 Pro 上刷入并回归**：解密 / Wi-Fi / 备份 / AOD / 锁屏 / 卡刷 /
   开机上锁全部过一遍。这一步不能省——OTA 只是把镜像写进分区，代替不了真机验证。
4. 生成发布包：
   ```bash
   python tools/make_release_zip.py --device nuwa \
     --image upstream/target/out/SW-v0.2.0-nuwa-v0.2.4/SW-v0.2.0-nuwa-v0.2.4.img \
     --out out/release
   ```
5. 建 GitHub 仓库、推送代码；建 tag `v0.2.4` 的 Release，上传
   `skyward-v0.2.4-nuwa-OS3.0.312.0.WMBCNXM.zip` 与 `SHA256SUMS`。
   release 不要勾 prerelease（`/releases/latest` 会跳过预发布）。
6. 手机里点「检查更新」：应显示「已是最新」（设备自报 `0.2.4`，最新 tag 也是
   `v0.2.4`），且不出现任何官方版本的提示。
7. 以后发 `v1.1.0`：改 `[update].release` → 重打包 → 生成新 zip → 发新 release，
   用户即可在 Recovery 内直接更新。

## 改了哪些代码（同步上游 0.3.1 时要一起带过去）

| 文件 | 改动 |
|---|---|
| `crates/skyward-config/src/lib.rs` | `Update` 增 `repo` / `release` 两个 `#[serde(default)]` 字段 |
| `crates/skyward-net/src/github.rs` | `LATEST` 常量 → `latest(repo)`；`ask()` 多收 `repo`；加两条单测 |
| `crates/skyward/src/updates.rs` | `Task::CheckSkyward` / `UpdateSkyward` 带 `repo`、`release`；repo 为空时不联网直接回答「已是最新」；`locate()` 的 User-Agent 用 `release` |
| `crates/skyward/src/ui.rs` | 从 `cfg.update` 取出 repo/release 传给 Task |
| `devices/nuwa.toml`（两份） | `[update]` 加 `repo` / `release` |

上游 0.3.1 **没有改** `skyward-net/src/github.rs` 与 `skyward-install/src/upgrade.rs`，
只动了 `updates.rs`(+98/−123) 和 `skyward-info`(+32/−21)，所以这批改动移植成本很低。

## 注意

- `github::ask` 是**未认证** GitHub API（按 IP 60 次/小时）。手动点检查频率低，一般无碍；
  公开发给很多人后若遇到 429，可换成自建的轻量更新 JSON 端点，不用动架构。
- `skyward_install::upgrade::slots()` 写死 `[Slot::A, Slot::B]`，**non-A/B 的 cepheus
  暂不启用在线更新**（它的 `repo` 保持为空），否则自更新会找不到槽位。
- 跨基线 OTA（0.2.0 基线 → 0.3.1 基线）本身只是换一张镜像；新镜像必须先在自己的
  13 Pro 上刷过并验证过再发布，不能把未验证镜像一键推给所有人。
- 同步 0.3.1 时若漏带这批改动，`LATEST` 会回到官方，用户更新一次之后又会误刷。
