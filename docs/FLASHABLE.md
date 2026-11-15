# 卡刷包（flashable ZIP）

把本项目的 recovery 镜像做成 ZIP，让已经刷了第三方 recovery（TWRP / OrangeFox 等）的人
可以直接在 REC 里「安装」刷入，不必连电脑用 fastboot。

> **当前产物**：`SW-v0.2.0-nuwa-v0.2.4-flashable.zip`，**仅适用于小米 13 Pro（`nuwa`）**。
> 小米 9（`cepheus`）走另一条分支与另一套镜像，不要用这个包。

## 怎么用

1. 前置：Bootloader 已解锁；手机里已是第三方 REC；建议先在该 REC 里备份原厂 `recovery`。
2. 把 ZIP 拷进手机（内部存储 / OTG 都行）。
3. REC 里「安装 / Install」→ 选这个 ZIP → 滑动确认。
4. 屏幕依次显示包信息、作者信息、四步进度，出现「刷入成功」后选**重启到 Recovery** 验证。
5. 首次进入需要输入锁屏密码解密 `/data`（这正是本项目要解决的问题）。

不进 REC 也行：解压出包内的 `recovery.img`，用
`fastboot flash recovery recovery.img` 刷入；这样只会写**当前槽位**，
而卡刷包会把 `recovery_a` 与 `recovery_b` 两个槽都刷上。

## 刷入时屏幕上显示什么

包内脚本按 Android 标准的 `update-binary` 协议输出 `ui_print` / `set_progress`，
下面是界面文本（`ui-preview.txt` 是脚本真机运行时导出的原文）：

```
==============================================
   Skyward Recovery  /  卡刷包
==============================================
  ----------------------------------------------
  版本 / Version   : SW-v0.2.0-nuwa-v0.2.4
  机型 / Device    : Xiaomi 13 Pro (nuwa)
  平台 / SoC       : Snapdragon® 8 Gen 2 Mobile Platform
  原厂底座 / Stock : OS3.0.312.0.WMBCNXM
  分区 / Target    : recovery_a + recovery_b (A/B 双槽 / both slots)
  镜像 / Image     : 100 MiB (104857600 bytes)
  SHA-256          : 59e568a0…4cfd35b24e4
  打包 / Built     : 2026-11-14
  ----------------------------------------------
  上游项目 / Upstream : kirklin/skyward
  上游作者 / Author   : Kirk Lin
  移植 / Port by      : Casper Lin
  许可 / License      : GPL-3.0
  ----------------------------------------------

  内置功能 / What is inside
  ----------------------------------------------
   · data 解密（TEE / KeyMint 服务链）
   · 息屏显示 AOD、锁屏（上滑 + 数字密码）
   · 60 fps 实时动态磨砂、三色电池
   · Wi-Fi / 备份恢复 / 安装 zip / MTP / 三语
  ----------------------------------------------

  刷入前请注意 / Before you flash
  ----------------------------------------------
   · 仅限 nuwa，其他机型会被拒绝
   · 需已解锁 Bootloader
   · 不会清除 /data 与内部存储
   · 建议先用本 REC 备份原厂 recovery
==============================================
```

之后是四步进度，每步都有标题与结果：

```
  [1/4] 校验机型 / Checking device
  [2/4] 取出镜像并校验 / Verifying image
  [3/4] 写入分区 / Flashing recovery      -> recovery_a / recovery_b
  [4/4] 回读校验 / Reading back           -> OK: recovery_a / recovery_b

==============================================
  刷入成功 / SUCCESS
  请重启到 Recovery 验证（不要直接重启系统）
  首次进入需输入锁屏密码解密 data
==============================================
  项目：kirklin/skyward
  移植：Casper Lin    上游：Kirk Lin
  许可：GPL-3.0     SW-v0.2.0-nuwa-v0.2.4
```

任何一步失败都会显示「刷入已中止 / ABORTED」加具体原因，并明确写出
「设备未做任何改动；可放心重启」，然后以非零码退出。

## 包结构

```
SW-v0.2.0-nuwa-v0.2.4-flashable.zip
├── META-INF/com/google/android/update-binary   0755  shell 脚本，真正干活
├── META-INF/com/google/android/updater-script  0644  edify 占位，兼容老 REC
├── recovery.img                               100 MiB，ZIP_STORED（不再压缩）
├── recovery.img.sha256                        镜像校验值
└── info.txt                                   完整中文说明（用法/作者/回退/风险）
```

REC 调用 `update-binary <api> <outfd> <zip>`；脚本往 `<outfd>` 写 `ui_print` 行显示文字、
写 `set_progress` 行走进度条。**注意用 `>>` 而不是 `>` 写 `/proc/self/fd/<n>`**：
传进来的是 pipe，`>` 带 `O_CREAT` 会报 `ENXIO`，界面会缺一半文字（真机实测踩过）。

## 四道安全校验

| 关卡 | 做什么 | 不通过 |
|---|---|---|
| 机型 | 读 `ro.product.device` / `ro.build.product` / `ro.product.vendor.device` / `ro.product.odm.device` / `/proc/cmdline`，必须命中 `nuwa` | 直接中止 |
| 分区 | 按 `by-name` 等三种路径找 `recovery_a` / `recovery_b`；读 `/sys/class/block/<dev>/size`，分区小于镜像则拒绝 | 直接中止 |
| 镜像 | 从 ZIP 取出后先比大小、再比 SHA-256 | 直接中止 |
| 写后 | 每个分区回读 100 个 1 MiB 块，SHA-256 与镜像比对 | 直接中止 |

此外：不碰 `/data`、不清内部存储、不 wipe cache；退出时清掉 `/tmp` 里的临时镜像。

## 重新生成

生成器 `tools/make_flashable_zip.py` 是参数化的，改了版本 / 机型后重跑即可
（与 `tools/make_test_zip.py` 一样属于本地工具：`tools/` 整目录在 `.gitignore` 里，
**不进版本库**；分发时 ZIP 自包含、不需要它。若想让它随仓库保存，挪到入库目录
或给 `.gitignore` 加一条例外即可）。

```bash
python tools/make_flashable_zip.py \
  --device nuwa \
  --image upstream/target/out/SW-v0.2.0-nuwa-v0.2.4/SW-v0.2.0-nuwa-v0.2.4.img \
  --out out/flashable
```

- `--device` 读 `devices/<codename>.toml`，取机型名、SoC、原厂版本和**是否 A/B**
  （A/B → 刷 `recovery_a` + `recovery_b`；非 A/B → 只刷 `recovery`）。
- `--version` 缺省取镜像文件名；手工合成的镜像名不含版本时要显式传。
- 镜像大小必须是整 MiB（脚本按 1 MiB 块写入和回读）。
- 产物是 ZIP 加同名 `.zip.sha256`。
- 生成器只**读取**镜像与配置，不改仓库里任何源码，也不依赖当前 git 分支。

## 回退

用 REC 里备份的原厂 `recovery` 分区，或从官方 full OTA 提取 `recovery.img` 后：

```bash
fastboot flash recovery stock-recovery-nuwa.img
```

系统与 `/data` 不受影响。更多细节见 [FLASH.md](FLASH.md)。

## 已知限制与注意

- **未在真实 TWRP / OrangeFox 上刷入验证过**：制作时手边只有小米 9（`cepheus`），
  在那台机上跑过完整 `update-binary`，验证了协议输出、命令兼容性与
  「非 nuwa 机器被拒绝且不写任何分区」，但**正向写入流程没在 13 Pro 上实刷**。
  分发前请先在 13 Pro 的 REC 里自己刷一次，确认出现「刷入成功」并能重启进 Recovery。
- 本包未签名。REC 若开启了「ZIP 签名校验」，需先关闭。
- 中文显示为方块是 REC 字体缺 CJK 字形；版本、机型、分区、SHA-256 都是 ASCII，
  不影响刷入与校验。
- 若确认机型正确却被「机型不符」拒绝，多半是第三方 REC 没暴露机型属性；
  解压出 `recovery.img` 改用 fastboot，或换一个 REC。
- 刷写过程中不要断电。脚本失败会中止并报错，不会留下半截镜像。
- 在 **Skyward 自己**里安装这个包属于自更新场景，其「保留当前 recovery」选项会
  影响结果；普通安装不受影响，但推荐在第三方 REC 里刷。

## 校验值

| 文件 | SHA-256 |
|---|---|
| `SW-v0.2.0-nuwa-v0.2.4-flashable.zip` | 见同目录 `.zip.sha256` |
| 包内 `recovery.img` | `59e568a02be1cd2881b9cbaad582d961392c76d044519eb0361604cfd35b24e4` |

镜像哈希与 [CHANGELOG.md](../CHANGELOG.md) 里已发版的 v0.2.4 镜像一致——
卡刷包**没有重新打包 recovery**，只是把同一张镜像换了个刷入方式。
