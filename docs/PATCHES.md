# 对上游源码的改动

本移植**不修改** Skyward 的架构与逻辑，绝大多数差异都在配置（`devices/nuwa.toml`）。
源码层面只有四类改动：为了在 Windows 上构建的两处可移植性补丁、两个新功能
（息屏显示、大写锁定）、版本号拆分，以及署名。下面逐条列出，便于对照上游 diff。

## 一、可移植性补丁（两处）

上游只在 Linux 上构建，打包器和配置校验用了一些 Unix 专有的路径语义。宿主机是 Windows
时需要以下两处最小改动。

### 1. `crates/skyward-bootimg/src/ramdisk.rs`

`ramdisk` 的 `unpack`、`c_path`、`set_time` 用的是 `std::os::unix` 的属性（uid/gid/mtime），
在 Windows 上无法编译。给这三处加 `#[cfg(unix)]`，并把测试模块改成 `#[cfg(all(test, unix))]`。

```rust
#[cfg(unix)]
pub fn unpack(...) -> Result<...> { ... }
```

影响：Windows 上无法解包 ramdisk（只有测试与调试会用到），**打包路径不受影响**——
打包只需要写入，不需要读回 uid/gid。

### 2. `crates/skyward-config/src/lib.rs`

配置校验用 `Path::is_absolute()` 判断「是不是绝对路径」。Windows 上 `"/system/bin/x"`
不算绝对路径（缺盘符），会导致 `[wipe]`、`[logd]`、`[[owners]]`、`[wlan]` 等段全部校验失败。

改为 `Path::has_root()`——它判断「是否根植于根」，Windows 与 Unix 上对 `/system/bin/x`
都返回真，语义正是配置想要的（Android 路径就是 `/` 开头的）。

共 5 处，每处一行：

```diff
- ensure!(Path::new(file).is_absolute(), ...);
+ ensure!(Path::new(file).has_root(), ...);
```

## 二、新功能（`crates/skyward-ui`）

这两项是真正的功能改动，全部在 UI crate 内，不触碰 drivers、安装、加密等逻辑。

### 息屏显示（AOD）

| 文件 | 改动 |
|---|---|
| `app.rs` | 新增 `AOD_LEVEL`(0.02)、`AOD_SPOTS`(4 个偏移)、`AOD_INK/DIM/FAINT/BLUE`、`AOD_HINT_MS`；`State.aod`；`Switch::Aod`；`App.aod_day`/`aod_since`；`panel()` 熄屏后把背光从熄灭瞬间的亮度渐变到 2%；`sleep()`/`wake()`/`touch()` 接入；`compose()` 转入 `compose_aod()`；`deadline()` 加 8 秒提示到期 |
| `fmt.rs` | `aod_date()`：三语带星期（`11月7日 星期六` / `Tue, Nov 7`） |
| `settings.rs` | `Settings.aod` 的读写与默认值 |
| `pages.rs` | 设置页「亮度」段新增「息屏显示」开关行 |

亮度渐变的方向是「以熄灭瞬间的亮度为起点」，不是「以 AOD 亮度为起点」——这样在
curtain 还没开始淡出的那几帧里，面板亮度严格等于熄灭前的值，不会有 f32 累加误差。

### 大写锁定

| 文件 | 改动 |
|---|---|
| `widgets.rs` | `Layer` 新增 `Locked` 变体与 `Layer::capital()`；`rows()`、Shift 键图标、切层键、键底色、字母大小写都改用它；锁定态在实心箭头下多画一条横杠 |
| `app.rs` | `App.shift_at` 记录上次点按时刻；`shift_tapped(now)` 实现状态机；`type_key`/`term_typed`/`edit_typed` 三处键盘入口统一走它（后两者改签名收 `now`） |

状态机：

```
Locked                      -> Letters     （点一次即解除）
Shifted 且 400ms 内再点      -> Locked      （双击锁定）
Shifted                     -> Letters     （单击取消一次性大写）
其余，400ms 内（刚解锁）      -> Letters     （连点两下解除后不会误锁）
其余                        -> Shifted
```

输入字符时清空 `shift_at`；只有 `Shifted` 会在输入后回落到 `Letters`，`Locked` 保持不变。

## 三、版本号（`crates/skyward-info`）

```rust
pub const UPSTREAM: &str = env!("CARGO_PKG_VERSION");   // 0.2.0，更新检查用这个
pub const PORT: &str = "0.2.1";                         // 本移植的第几版
pub const VERSION: &str = concat!("SW-v", env!("CARGO_PKG_VERSION"), "-nuwa-v0.2.1");
```

`VERSION` 只用于展示（About 页、MTP 设备名、内核日志）；`crates/skyward/src/updates.rs`
里三处调用改用 `UPSTREAM`，因为 GitHub 上只有上游的 release，复合版本号既解析不了也不该
被拿去比较。

## 四、署名（`crates/skyward-ui`）

上游的 About 页在版本号下面有一行作者：`© 2026 Kirk Lin`。本移植在它**下面**加一行
自己的署名，原行一字未改——GPL-3 要求保留原版权声明，追加而不是替换。

| 文件 | 改动 |
|---|---|
| `widgets.rs` | 新增 `port_line()` 与 `PORT_BELOW_DP`(36.5)；`author_line()` 与它共用 `hero_line()`，只是垂直位置不同 |
| `app.rs` | `HeroArt` 新增 `port` 字段，与 `author` 一同绘制、一同随滚动淡出缩放 |
| `pages.rs` | 新增 `PORT` 常量 `© 2026 Lin Zixi`；许可页在原文之上新增「修改说明」段（中/繁/英），写明修改部分版权与三处改动 |

许可页那段是 GPL-3 第 5(a) 条对修改版本的要求：显著声明改动过、以及改动的时间。它放在
「原文」段（`LICENSE_ORIGINAL`）之前，原文本身不动。

## 与其它移植版本的关系

酷安上有另一个面向 nuwa 的移植（`pubglite55/skyward-for-nuwa`），基于上游 311 版，
多一个 AOD（三种时钟样式循环），少双系统等功能。本移植的 AOD 与它是**各自实现**：功能
需求相同（暗屏低亮度显示时间、防烧屏、触摸唤醒），代码按本项目的状态机重写，未取用其
实现，因此不涉及署名义务。功能差异的对比结论见 [README.md](../README.md) 的「本移植相对
上游做了什么」。
