# 新机型适配交接

> **给新会话的 AI**：这份文档是接着本项目适配另一台机型的起点。按 §0 → §8 顺序做，
> 不要跳过 §0。文中所有路径都相对仓库根 `C:\Users\LZX\WorkBuddy\Skyward`。
> 动手前先读 `.workbuddy/memory/MEMORY.md` 与最近一天的 `.workbuddy/memory/YYYY-MM-DD.md`，
> 那里记着本项目历轮的真实踩坑，比任何文档都新。

**交接时的状态（2026-11-11）**

| 项 | 值 |
|---|---|
| 当前机型 | 小米 13 Pro，codename `nuwa`，SM8550，Android 16 / `OS3.0.312.0.WMBCNXM` |
| 当前版本 | `SW-v0.2.0-nuwa-v0.2.4`，提交 `f8a39e5`，标签 `SW-v0.2.0-nuwa-v0.2.4` |
| 已刷入镜像 sha256 | `59e568a02be1cd2881b9cbaad582d961392c76d044519eb0361604cfd35b24e4` |
| 真机验收 | 启动/显示/触摸、data 解密、Wi-Fi、备份、AOD、锁屏、60 fps 动态磨砂全部通过 |
| 测试基线 | 交叉编译推真机跑：**357 通过 / 1 失败**（`lang::tests::the_typeface_draws_every_text_the_pages_write` 要读源码树，设备上没有；属环境限制，不是回归） |
| 回退点 | 标签 `SW-v0.2.0-nuwa-v0.2.3` → `0fd7166`，`…v0.2.2` → `730aafb` |

---

## 0. 先问用户，再动一行代码

下面这些不问清楚，后面全部是白做。**尤其前两题是架构决策，别替用户拍板。**

| # | 要问的 | 为什么 |
|---|---|---|
| 1 | 新机型的 **codename、型号、SoC 平台、当前系统版本号** | 决定配置能不能复用、参考哪个机型 |
| 2 | **要把新机型做成独立分支/独立副本，还是让同一个仓库同时支持多机型？** | 本项目的版本号、`README`、`build.sh`、`skyward-info` 全部写死 `nuwa`（清单见 §2）。同仓多机型要参数化 8 处以上，并会动到已验证的 nuwa 路径；**默认推荐独立分支或仓库副本** |
| 3 | 该机是 **A/B 分区**吗？**recovery 是独立分区**还是在 `boot` / `init_boot` 里？ | 当前整条打包链假设「独立 recovery 分区」。若 recovery 在 boot 里，`build.sh` 的 `pack` 流程**不适用**，得先解决底座结构（可用 `skyward-pack boot` 合并 kernel + ramdisk） |
| 4 | Bootloader 解锁了吗？设备在手边吗？能开 USB 调试吗？有 root 吗？ | 采集脚本需要 adb；root 能直接 `dd` 备份原厂 recovery |
| 5 | 拿得到该版本的**官方 full OTA zip**（或已提取的 `recovery.img` + sha256）吗？ | `[stock]` 三件套是打包的硬前提，见 §4 |
| 6 | 期望的功能边界：只要能启动 + 解密 `/data`，还是要 AOD/锁屏/Wi-Fi/备份全套？ | 决定验收清单取舍，也决定要不要把本项目那些 UI 补丁一起带过去 |

**顺带确认交付规则**（项目约定，覆盖本文件所有内容）：默认只做「同版本优化」——
改代码、文档、跑测试、渲染核对都行，但**不升版本号、不 `bash build.sh` 打包、不 git 提交、
不打标签**；只有用户明确说「打包 / 刷机 / 发版 / 升版本」时才走完整交付流程。一轮做完停下等确认。

---

## 1. 三十秒看懂这个项目

不是从零写 recovery，而是**拿原厂 `recovery.img` 做底座**：保留它的 kernel、init、adbd、
SELinux 策略和库，只把 `system/bin/recovery` 换成我们编的 `skyward`，再往 ramdisk 塞进
`device.toml`、两个字体、`system-init` 和几个 rc/json，最后补一个未签名的 AVB footer。
所以显示、触摸、存储这些底层沿用原厂实现，**绝大多数机型差异都集中在一份 TOML 配置里**，
Rust 源码层面基本不用改。

```
build.sh
 ├─ binaries  cargo ndk → skyward / skyward-pack / system-init(静态)
 └─ pack
     ├─ skyward-pack stock   --config devices/<codename>.toml
     │    └─ 从官方 full OTA 流式抽 recovery.img，按 [stock].recovery 校验 sha256
     │       缓存到 upstream/target/stock/<codename>/recovery.img
     └─ skyward-pack recovery --stock … --binary … --config …
          └─ 解 ramdisk → 换 recovery → 塞配置/字体 → 改 prop.default
             → 重压 → 按底座 AVB footer 得出的分区大小补脚 → <VERSION>.img
```

**目录地图**

| 路径 | 作用 | 是否入库 |
|---|---|---|
| `devices/<codename>.toml` | **打包真正读的配置**（`build.sh` 的 `--config`） | 是 |
| `upstream/devices/*.toml` | 上游 21 个机型的原始配置，是最好的模板库 | 是 |
| `upstream/devices/nuwa.toml` | **已被本项目改成和 `devices/nuwa.toml` 完全一致**（228 行、8 个 service） | 是 |
| `refs/` | 上游原始 `nuwa.toml`(152 行)、官方包、第三方移植源码，**本地参考资料** | 否（`.gitignore`） |
| `device/` | 从真机采集的 321 个文件：props、分区、模块、init rc、bin 清单 | 是 |
| `device/collect.sh` | 采集脚本，**适配新机型第一个要跑的东西** | 是 |
| `tools/patch_nuwa.py` | 一次性脚本：把 ishtar 的 TEE 链插进上游 nuwa 配置。**别当通用工具复用** | 是 |
| `tools/` | NDK r29、platform-tools、lz4.exe 等 | 否（`.gitignore`） |
| `out/`、`logs/`、`upstream/target/`、`build/` | 渲染图、构建日志、产物 | 否 |

> `refs/`、`tools/`、`out/` 都不进 git。换一台电脑或全新 clone 时它们不存在，
> 需要先从本机拷过去或重新下载依赖（版本见 `docs/BUILD.md`）。

---

## 2. 机型名与版本号写死在哪（逐处改）

`nuwa` 这个字符串不只出现在配置里。新机型必须逐处核对：

| 文件 | 现在的内容 | 新机型怎么处理 |
|---|---|---|
| `build.sh` | `DEV="nuwa"`（第 21 行）+ 开头注释 | 改成新 codename；同仓多机型则要改成 `DEV="${DEV:-nuwa}"` 或读参数 |
| `devices/nuwa.toml` | 228 行配置 | **新建** `devices/<codename>.toml`，旧的留着当回退与参照 |
| `upstream/devices/nuwa.toml` | 与上面字节相同 | 独立分支方案下不要动；同仓方案下也要为新机型建一份 |
| `VERSION` | `SW-v0.2.0-nuwa-v0.2.4` | 新机型建议从 `SW-v0.2.0-<codename>-v0.1.0` 起算（上游基线升级时项目也会重新计数，见 CHANGELOG）；**具体命名先跟用户确认** |
| `upstream/crates/skyward-info/src/lib.rs` | `PORT = "0.2.4"`、`VERSION = concat!("SW-v", …, "-nuwa-v0.2.4")` 及其注释 | 改 codename 与序号；同仓多机型建议把 codename 用 `env!` / build.rs 注入，别再写死 |
| `README.md` | 标题、当前版本表、功能表、版本列表、哈希示例、fastboot 示例 | 全文替换；版本命名规则段也要改 |
| `CHANGELOG.md` | 整本都是 nuwa 的历史 | **保留历史不动**，顶部新增新机型条目；别像曾经那样把旧版本段落重写成未发布的内容 |
| `docs/BUILD.md`、`docs/FLASH.md` | 示例镜像名、stock 备份名、`--config` 路径 | 全文替换 |
| `docs/PATCHES.md` | 版本号常量示例、机型说明 | 只改常量示例与机型描述，UI 改动说明可保留 |
| `upstream/crates/skyward/src/main.rs:702` | 启动日志里的 codename 来自 `cfg.device.codename` | **不用改**，配置给什么就打什么 |

注意 `skyward-info` 的 `UPSTREAM` 常量（更新检查用）必须保持上游 `CARGO_PKG_VERSION`，
`crates/skyward/src/updates.rs` 已经改用 `UPSTREAM` 比较版本，别把复合版本号塞回去。

---

## 3. 采集：先把这台机器「读」出来

手机在**正常系统**里、USB 调试已授权，在项目根执行：

```bash
ADB=tools/platform-tools/adb.exe bash device/collect.sh
```

脚本会打印 `ro.product.device` 与 `ro.build.version.incremental`，把结果写到 `device/`：
文本文件（`props.txt`、`byname.txt`、`blockdev.txt`、`mounts.txt`、`lsmod.txt`、
`modules.txt`、`touch.txt`、`hal_bins.txt` 等）与扁平化的目录
（`vendor__etc__init/…`、`vendor__etc__vintf/…`、`odm__etc__init/…`、
`vendor__lib__modules/…`、`odm__firmware/…`）。

**每个采集结果对应配置哪一段**：

| 采集物 | 用来确定 |
|---|---|
| `props.txt` | `[device].os_version_prop`（本项目是 `ro.mi.os.version.incremental`）、`ro.product.device`、Android 版本、分辨率/density 线索、`ro.boot.slot_suffix` 判断 A/B |
| `byname.txt` / `blockdev.txt` | 分区名 → `[backup].efs`、`[partitions].mounted`、是否有独立 `recovery` |
| `lsmod.txt` / `modules.txt` | 实际加载的内核模块 → `[modules].load`（触摸驱动）、`[wlan].modules` |
| `vendor__lib__modules/` | `modules.load`、`modules.dep`；确认驱动名与所在分区 → `[modules].sources` |
| `odm__etc__init/*.rc`、`vendor__etc__init/*.rc` | `[[service]]` 的 `path`/`user`/`groups`/`capabilities`/`oneshot`/`ready_property`；`[[owners]]` 的 chown/chmod 行 |
| `hal_bins.txt` / `odm_bin_hw.txt` | TEE 五个 HAL 的真实路径（不同机型路径可能变） |
| `touch.txt` | 触摸 sysfs 节点 → `[[owners]].paths`、`[touch].writes` |
| `odm__firmware/`、`firmware_mnt.txt` | 触摸/WLAN 固件位置 → `[wlan].firmware` |
| `mounts.txt` | 分区挂载点与文件系统（`/data` 是 f2fs 还是 ext4） |

> 采集脚本在 Windows Git Bash 上已处理路径转换（`MSYS_NO_PATHCONV=1`、
> `MSYS2_ARG_CONV_EXCL='*'`，pull 时用 `cygpath -w`）。**别删这几行**，否则 `/vendor/...`
> 会被改写成 `C:/Program Files/Git/vendor/...`。

---

## 4. 写 `devices/<codename>.toml`

### 4.1 先找参考，别从空白开始

```bash
ls upstream/devices/          # 21 个机型
```

优先顺序：

1. **上游有该机型** → 直接拿它当底稿（注意上游多数机型只有 1~3 个 `[[service]]`，
   **不含 TEE 链，解不了 `/data`**）。
2. **同 SoC 平台且已验证的机型** → 本项目 `nuwa` 的 TEE 链是从
   `upstream/devices/ishtar.toml`（小米 13 Ultra，同为 SM8550）搬来的；
   `ishtar`(7 services)、`nezha`(8 services) 是配置最全的两份，值得逐段对照。
3. **本机采集结果** → 修正上游配置里与新机不符的路径、模块名、节点。

现有 `devices/nuwa.toml` 共 228 行，比上游原版 `refs/nuwa.upstream-orig.toml`（152 行）
多出的 76 行就是 TEE 链与备份/日志段，可以当作「完整版长什么样」的样板。

### 4.2 逐段说明

配置 schema 在 `upstream/crates/skyward-config/src/lib.rs`（1028 行，含字段注释与校验）。
除 `[device].codename` 外几乎都有 `#[serde(default)]`，但**功能上有几段是必需的**：

| 段 | 作用 / 从哪取 | 漏了会怎样 |
|---|---|---|
| `[device]` | `codename`、`brand`、`model`、`ab`、`soc`(三语文案)、`os_version_prop` | `ab` 错会找错槽位；`os_version_prop` 错影响版本显示 |
| `[display]` | `width`/`height`/`density`（+可选 `diagonal`、`corner_radius`）。取**面板原生模式**，不是系统当前降分辨率后的值 | 版式全部错位、触摸坐标缩放错 |
| `[modules]` | `sources`（`vendor_boot`/`vendor_dlkm`/`vendor`…）、`load`（触摸驱动，本项目 `fts_touch_spi`） | 触摸没反应 |
| `[[service]]` | 按 init rc 逐个抄。**TEE 五件套**：`qseecomd` → `keymint-service-qti` → `hwservicemanager` → `gatekeeper@1.0-service-qti` → `boot@1.2-service`；顺序不能乱，各自带 `ready_*` | **`/data` 解不了密**，这是整个移植的核心 |
| `[[owners]]` | 从 `/odm/etc/init/vendor.xiaomi.hw.touchfeature@1.0-service.rc` 的 chown/chmod 行抄；只保留本机驱动确实有的节点 | 触摸服务起来但打不开节点 |
| `[touch]` | 驱动起来后要写的 sysfs 值 | 某些屏手势/触控异常 |
| `[wlan]` | `modules`、固件 `firmware`(含 `partition = "persist"` 的 MAC)、`interface`、`supplicant`、`service`、`probe` | Wi-Fi 不通 |
| `[haptics]` | `click`(波形编号)、`resonance` 节点、`strength` | 震动无声（锁屏/AOD 反馈失效） |
| `[backup]` | `efs`（基带/EFS 分区，本项目 5 个）、`data_skipped` | 备份会漏掉 IMEI 或备份到 TEE 私有目录 |
| `[wipe]` | `logs` 目录清单 | 「导出日志」没有内容可选 |
| `[update]` | 厂商 OTA 服务器 | 更新检查不可用（非致命） |
| `[stock]` | **见 4.3** | 打包直接失败 |

**校验方式**：`skyward-pack recovery` 启动时会 `DeviceConfig::load(&config)?`，字段类型、
绝对路径（Windows 下用 `Path::has_root()`）、模块来源都会当场校验。改完配置先跑一次
`bash build.sh pack` 让它报错，比刷进去再排查便宜得多。

### 4.3 `[stock]` 三件套（最容易卡住的一步）

```toml
[stock]
version  = "OS3.0.312.0.WMBCNXM"                      # 与手机上一致的系统版本
package  = "https://…/nuwa-ota_full-…zip"             # 官方 **full** OTA zip，不是增量包
recovery = "04431f15…"                                # 该包内 recovery 分区的 sha256
```

`skyward-pack stock` 会**流式**从 `package` URL 里只抽 `recovery` 分区，并拿
`stock.recovery` 做 sha256 强校验（`main.rs:172`）。拿到这个哈希的循环是：

1. 下载 full OTA zip，用 `payload_dumper` 解出 `recovery.img`；
2. `sha256sum -t recovery.img` → 填进配置；
3. 跑 `./upstream/target/release/skyward-pack stock --config devices/<codename>.toml --out …`
   验证；
4. **实用技巧**：如果一时算不出，随便填一个值跑一次，报错信息里会打印包内真实的
   `has the SHA-256 {sum}`，抄回来即可。

另外：

- 底座必须是**原厂 stock recovery**；打包器检测到 `FFiles` / `twres` 会拒绝
  （"not a stock recovery"），不能拿 TWRP 之类的当底座。
- 分区大小不是硬编码 100 MiB：`partition_size()` 读底座 AVB footer 得出，新机型分区多大就出多大。
- 底座与**当前手机固件版本**要匹配：版本不对可能卡在开机 logo（见 `docs/FLASH.md`）。
- 打包器会自动从 `prop.default` 去掉 `ro.build.version.release` 与两条 security patch 属性
  （KeyMint 要靠这些向 TEE 报构建版本），并加 `ro.secure=0` 等四行。这段逻辑通用，别改。

---

## 5. 构建、刷入、验收

### 5.1 构建

```bash
bash build.sh             # 全量：二进制 + 底座 + 打包
bash build.sh binaries    # 只编二进制
bash build.sh pack        # 只重打包（改 TOML 时用这个，几十秒）
```

环境依赖与 Windows 坑全在 `docs/BUILD.md`。要点：

- 宿主 `cargo check -p skyward-ui` 在 Windows 会因 `rsproperties`/`rustix`/`libc` 失败，
  **必须用** `cargo ndk -t arm64-v8a --platform 30 …`，且只有 `-p skyward` 才是主程序。
- `ANDROID_NDK_HOME` 要 `C:/…` 风格（`cygpath -m`），`adb`/`fastboot` 的参数路径要防 MSYS 转换。
- `lz4.exe` 必须是能产出 **legacy 帧**的外部 CLI，纯 Rust 实现不能替代。
- **改了 `devices/*.toml` 必须重新打包**：配置是打进 ramdisk 的，不是运行时读外部文件。
- 每轮编译务必确认输出里有 `Finished` 或检查二进制 mtime —— 曾经编译失败但 `tail -3`
  没看到错误，推了**旧二进制**跑出「全部通过」的假结果。

打包后核对（这几条都做过才叫交付）：

```bash
sha256sum -t upstream/target/out/<VERSION>/<VERSION>.img
./upstream/target/release/skyward-pack cat <IMG> system/bin/recovery | sha256sum -t
sha256sum -t upstream/target/aarch64-linux-android/release/skyward   # 两者必须相同
strings upstream/target/aarch64-linux-android/release/skyward | grep -m1 "SW-v0.2.0-"
```

### 5.2 刷入

命令见 `docs/FLASH.md`。**先备份原厂 recovery**，再：

```bash
adb reboot bootloader
fastboot flash recovery <VERSION>.img
fastboot reboot recovery
```

刷完立刻核对（这一步能抓出"刷错镜像"）：

```bash
adb shell "head -1 /skyward/run/recovery.log"   # 应显示 <codename> <VERSION> recovery up
adb shell "sha256sum /system/bin/recovery"     # 应等于本地 skyward 二进制哈希
```

> **刷机会清掉设备 `/tmp/`**：测试字体和预览配置每次都要重推。
> ```bash
> adb shell "mkdir -p /tmp/fonts"
> adb push upstream/resources/fonts/MiSansVF.ttf /tmp/fonts/
> adb push upstream/resources/fonts/KirkMono-Regular.ttf /tmp/fonts/
> adb push devices/<codename>.toml /tmp/<codename>.toml
> ```
> 别照抄 `/tmp/nuwa.toml`，新机型要换成自己的文件名。

### 5.3 验收清单（照着逐项打勾）

| 项 | 判定方式 | 不通过先查 |
|---|---|---|
| 启动 / 显示 | 开机日志 `… recovery up`，UI 正常显示 | 底座版本、AVB footer、分区结构 |
| 触摸 | 能滑动、能点按 | `[modules].load`、`[[service]]`、`[[owners]]`、`[touch]` |
| **data 解密** | 输锁屏密码后 `/data` 以读写挂载（本项目是 f2fs，识别出 0/10/999 三个用户） | **TEE 五件套是否全起**、`ready_*` 顺序、`prop.default` |
| 密码错误提示 | 报 `the secure world does not recognise that credential` 说明**链路是通的**、只是密码不对；报别的错才是链路问题 | 同上 |
| Wi-Fi | WPA3/SAE 连上、拿到 IPv4 + IPv6 | `[wlan]` 模块、固件路径、MAC 分区、`interface` |
| 备份 | 产出镜像且 `modemst1` 的 sha256 与清单一致 | `[backup].efs` 分区名 |
| AOD / 背光 | 15 s 后亮度降到约 2%，轻触唤醒 | 背光节点由代码枚举 `/sys/class/backlight`、优先 `panel0-backlight`；节点名不同要改 `skyward/src/ui.rs:95` |
| 震动 | 键盘/长按有反馈 | `[haptics]`；节点不在 `qcom-haptics` 就要换 |
| 锁屏状态机 | 开机先落锁屏、电源键上锁且保持亮屏、超时才进 AOD | 这是本项目自己加的功能，**上游没有**；独立分支方案要确认带过去了 |
| 全量 UI 测试 | 交叉编译测试二进制推真机跑，基线 357 通过 / 1 环境失败 | 见 §6 |

调试命令：

```bash
adb shell dmesg | grep -i skyward
adb shell cat /metadata/skyward/settings.json
adb shell cat /sys/class/backlight/panel0-backlight/brightness
```

**版式核对只能用 `skyward-preview`**（交叉编译推到 `/tmp` 跑，recovery 里没有 fb0，截不了屏）：

```bash
cargo ndk -t arm64-v8a --platform 30 build --release -p skyward-preview
adb push upstream/target/aarch64-linux-android/release/skyward-preview /tmp/
adb shell "SKYWARD_FONT=/tmp/fonts/MiSansVF.ttf /tmp/skyward-preview /tmp/pages \
  --config /tmp/<codename>.toml --lang zh-Hans"
```

预览工具还有 `--aod`、`--lock`、`--lock-about`、`--bench-blur` 等开关。
预览里手工改过 `App.state` 之后**必须 `refresh()`**，否则 `render()` 复用旧帧。

---

## 6. 测试与踩坑（血泪，别重踩）

```bash
cd upstream
export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'
export ANDROID_NDK_HOME="$(cygpath -m /c/Users/LZX/WorkBuddy/Skyward/tools/android-ndk-r29)"
cargo ndk -t arm64-v8a --platform 30 test --release -p skyward-ui --no-run
cd ..
A=./tools/platform-tools/adb.exe
B=$(ls -t upstream/target/aarch64-linux-android/release/deps/skyward_ui-* | grep -v '\.d$' | head -1)
$A push "$B" /tmp/skyward-ui-test && $A shell chmod 755 /tmp/skyward-ui-test
$A shell "SKYWARD_FONT_DIR=/tmp/fonts /tmp/skyward-ui-test 2>&1 | tail -20"
```

- 全树行尾必须是 **LF**（`.gitattributes` 里 `* text=auto eol=lf`）。Windows 上写完
  `.rs`/`.md`/`.sh` 后用 `git ls-files --eol | grep w/crlf` 查一遍。
- **编译失败时别推旧二进制**：先确认 `Finished`，或对比
  `ls -l --time-style=full-iso <测试二进制>` 的 mtime。
- 新建 `adb shell` 会重新枚举 USB，被 UI 当成插线事件；观察 AOD 必须在**单条连接**里
  `sleep` 之后再读，不要反复开 shell。
- `lang::tests::the_typeface_draws_every_text_the_pages_write` 会
  `read_dir(CARGO_MANIFEST_DIR/src)`，设备上没源码树 → `NotFound`，harness 记为
  **failed**（历史文档里委婉写成「1 无法运行」）。这是长期环境限制，不是回归。
- `dev` 与 `release` 产物别混用；`build.sh` 用 release。
- 标签即回退点：每发一版新建标签，**旧标签一律不动**。

---

## 7. 故障速查

| 症状 | 最可能的原因 |
|---|---|
| 卡开机 logo / 起不来 | 底座 `recovery.img` 与手机固件版本不匹配、ramdisk 压缩方式不同、分区结构不同（尤其 recovery 不在独立分区时） |
| 能进界面但 `/data` 挂不上 | TEE 五件套缺一个或 `ready_*` 顺序错；HAL 路径在本机不一样；`prop.default` 的构建版本属性没处理干净 |
| 触摸完全没反应 | `[modules].load` 没加载触摸驱动；`[[service]]` 没起；`[[owners]]` 权限没给；`sources` 指错分区 |
| 背光不变 / AOD 不亮 | `/sys/class/backlight` 下没有 `panel0-backlight` 且枚举回退不对（`skyward/src/ui.rs`） |
| 无震动 | `[haptics]` 的 `resonance` 节点、`click` 波形编号与本机驱动不符 |
| Wi-Fi 连不上 | `[wlan].modules` 芯片型号不对；固件/ MAC 路径不对（注意 `persist` 分区） |
| 「开机直接进功能界面」/「电源键直接 AOD」 | 若本分支已带上本项目那套锁屏，说明刷的是 v0.2.3 或更早的镜像，先用 `sha256sum /system/bin/recovery` 核对；nuwa 的 v0.2.4 已修这两个状态机 bug |
| 打包报 `not a stock recovery` | 底座是第三方 recovery，换官方 full OTA 里的 |
| 打包报 sha256 不符 | `[stock].recovery` 填错，或 OTA 包版本与配置 `version` 不一致 |
| Windows 上 `cargo check` 一堆 Unix 错误 | 正常，改用 `cargo ndk`；见 `docs/BUILD.md` |

---

## 8. 交付前核对

- [ ] §0 的六个问题都有答案，尤其**架构方案**是用户明确选的
- [ ] `devices/<codename>.toml` 通过 `DeviceConfig::load` 校验（`bash build.sh pack` 没报配置错）
- [ ] `[stock]` 的 version / package / recovery 三项自洽，`skyward-pack stock` 能抽并校验通过
- [ ] §2 清单里所有 `nuwa` 硬编码要么已改、要么确认是历史条目不该动
- [ ] 版本号按 `SW-v<上游>-<codename>-v<序号>`，起算序号**与用户确认过**；`CHANGELOG` 顶部新增条目、**历史段落不重写**
- [ ] 镜像 sha256、图内 `system/bin/recovery` 哈希、本地二进制哈希三者核对过
- [ ] `fastboot flash` 后启动日志显示新 codename 与新版本，设备二进制哈希一致
- [ ] §5.3 验收表逐项打勾，真机 UI 测试结果记录（通过数 + 那条已知失败）
- [ ] 刷机后 `/tmp/fonts` 与 `/tmp/<codename>.toml` 已补回，方便下一轮
- [ ] **用户明确说了「打包 / 刷机 / 发版 / 升版本」**才提交、打标签；否则停在这里等确认

---

**相关文档**：[BUILD.md](BUILD.md)（依赖与构建坑）、[FLASH.md](FLASH.md)（备份/刷入/回退）、
[PATCHES.md](PATCHES.md)（相对上游的源码改动全清单）、[README.md](../README.md)（功能与实测）、
[CHANGELOG.md](../CHANGELOG.md)（逐版历史）。
