# 对上游源码的改动

本移植**不修改** Skyward 的架构与逻辑，绝大多数差异都在配置（`devices/nuwa.toml`）。
源码层面只有四类改动：为了在 Windows 上构建的两处可移植性补丁、三个新功能
（息屏显示、锁屏、大写锁定）、版本号拆分，以及署名。下面逐条列出，便于对照上游 diff。

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

这三项是真正的功能改动，全部在 UI crate 内（锁屏的密码哈希在 `skyward-crypto`），
不触碰 drivers、安装、加密等逻辑。

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

### 锁屏

面板熄灭即上锁，再亮起来先看到锁屏；先上滑，设了密码才接着输密码（可在设置里另设 4–8 位）。
锁屏的背景是把当前页面缩到六分之一做两次方框模糊、颜色推回饱和、再拉回全屏而成的，
光度一点不压，上面也不铺任何东西。字色由这块玻璃自己决定：读一次平均亮度，亮的用深墨、
暗的用白字；也可以在设置 →「锁屏」→「锁屏文字」里定死，选跟随系统、浅色（深色字）或
深色（白色字），定死的颜色不看玻璃，只按选项。

时钟下面那一行是电量：一个与百分比同色的电池（圆角外壳 + 右侧极头，内部按电量填充），
左图标右数字，两者一起居中——「锁屏电量」开关关掉就只剩百分比。

颜色是三种状态三种颜色，锁屏与息屏显示用同一套：

| 状态 | 颜色 | 含义 |
| --- | --- | --- |
| 平常（放电、≥20%） | `AOD_BLUE` `#5488F8` | 还剩多少 |
| 插线 | `#1DCD3A` | 正在充，电池内多一个黄闪电 |
| 低于 20% 且没插线 | `#E64B3B` | 该插线了 |

阈值直接取状态栏的 `LOW_BATTERY`(20)，不再单设一个。这三种颜色不看锁屏字色，也不看
玻璃明暗——蓝色在浅玻璃上是 3.3:1 的对比，字号 14dp 够读；换成黑白反而会让同一块电池
在浅色页面和深色页面上各是一个样子，AOD 又是第三个样子。

同一行也画在息屏显示上：`battery_ink(level, plugged)` 一处定色，`compose_battery()` 一处
摆位，锁屏与 AOD 的几何、三态、闪电、开关完全共用，只有落点不同（`cy+112` 与 `cy+126`）。
AOD 的那一行跟着每分钟的位移走，`state.lock_battery` 关了它俩一起回到光秃秃的百分比。

闪电用黄色（`#FFD60A`）而不是电池色：充电既不是电量也不是警告，第三色不用解释，而且黄
在绿填充和淡绿空槽上都读得出来。它只有一个形态，深浅模式共用，用两段凸多边形（`LOCK_BOLT_TOP`/
`LOCK_BOLT_FOOT`）拼成，对齐电池**主体**中心。充电时电池空出来的部分填得实一点
（`0x52` 而不是 `0x30`），给黄色一个能站住的底——电量极低时填充只剩一丝，闪电其实整根
都落在空槽上。

| 文件 | 改动 |
|---|---|
| `skyward-crypto/src/pin.rs` | 新增：`allowed()`（4–8 位纯数字）、`hash()`（scrypt N=2^13/r=8/p=1，16 字节 `/dev/urandom` 盐，输出 `s1$<位数>$<盐>$<哈希>`）、`check()`（常量时间比较）、`length()`、`read()`（一行只有一个解析器，`length` 由它派生，避免两处格式判断走偏） |
| `skyward/src/pin.rs` | 新增：`Worker`，把拉伸放到后台线程，`pipe2(O_CLOEXEC\|O_NONBLOCK)` 唤醒主循环，`Done::{Saved,Checked}` 回传（与 `Decryptor` 同一个范式） |
| `skyward-ui/src/app.rs` | `State.lock`/`lock_code`/`code_entry`；`Switch::Lock`、`Action::ForgetCode`、`Screen::SetCode`、`Effect::SaveCode`/`CheckCode`；`State.lock_ink`、`Screen::LockInk`、`Action::LockInk`；`App.locked`/`pulled`/`lock_from`/`lock_pressed`/`lock_typed`/`lock_wrong`/`lock_asking`/`lock_backdrop`；`LOCK_SLID`(0.6)、`LOCK_PULL_DP`(220)、`LOCK_WRONG_MS`(4000)、`LOCK_BLUR_SHRINK`(6)、`LOCK_BLUR_RADIUS`(3)、`LOCK_COLOUR`(1.55)、`LOCK_INK_WAYS`(3)/`LOCK_INK_FOLLOW`/`LOCK_INK_LIGHT`/`LOCK_INK_DARK`、`LOCK_LIGHT`(128)、`LOCK_MARK`/`LOCK_MARK_DIM`（亮玻璃上的深墨）、`LOCK_BATTERY_W_DP`(26)/`LOCK_BATTERY_H_DP`(12.5)/`LOCK_BATTERY_NUB_DP`/`LOCK_BATTERY_PAD_DP`/`LOCK_BATTERY_STROKE_DP`/`LOCK_BATTERY_GAP_DP`、`LOCK_BATTERY_GREEN`(`#1DCD3A`)/`LOCK_BATTERY_LOW`(`#E64B3B`)/`LOCK_BATTERY_LOW_LEVEL`(20，与状态栏同)、`LOCK_BOLT_YELLOW`(`#FFD60A`)、`LOCK_BOLT_W_DP`/`LOCK_BOLT_H_DP`/`LOCK_BOLT_TOP`/`LOCK_BOLT_FOOT`、`Switch::LockBattery`；`battery_ink()` 定颜色（平常 `AOD_BLUE`、插线绿、低电红，锁屏与 AOD 同一个函数）、`battery_body()` 画那块电池（含充电时的闪电）、`compose_battery()` 把它与百分比摆成一行——锁屏与 AOD 共用这一支笔；`sleep()`/`lock_now()` 走 `lock_put()` 上锁（按键模式下跳过）、`touch()` 与 `key()` 在锁屏处收口、`lock_touch()`/`ask_code()`/`lock_key()`/`buzz()`/`code_typed()`/`unlock_panel()`/`code_saved()`/`code_checked()`/`forget_code()`；`compose_lock()` 及其两套版式（背景为 `Canvas::blurred()` 加 `saturate()` 的结果，一点不压暗）；`deadline()` 加错误提示到期 |
| `skyward-ui/src/canvas.rs` | 新增 `blurred(factor, radius)`：按 `factor` 缩块平均、在缩图上做两次方框模糊、再双线性拉回全屏；`saturate(amount)`：把每个像素推离它自己的灰（0 为单色，1 不变，大于 1 加深），补回模糊洗掉的色；`luma()`：整块画布的平均亮度（Rec.709 权重），决定锁屏用深字还是白字。锁屏每次上锁只算一次，结果留在 `App.lock_backdrop` |
| `skyward-ui/src/widgets.rs` | `keypad()` 的确认键文字改为入参（原来的「解密」写死），并拆出只算不画的 `keypad_rects()` 供锁屏命中；新增 `code_dots()` |
| `skyward-ui/src/pages.rs` | 新页 `set_code()`（两步输入）与 `lock_ink()`（三种字色，`Trail::Check` 单选）；设置页「亮度」段之下新增「锁屏」段（开关 + 锁屏电量 + 解锁密码 + 锁屏文字行） |
| `skyward-ui/src/settings.rs` | `Settings.lock`/`lock_code`/`lock_ink`/`lock_battery` 的读写；`lock_ink` 以 `follow`/`light`/`dark` 三个名字存，读回来认不出的就保持原状 |

几处刻意的决定：

- **上锁只看 `sleep()`**，开机不上锁——开机进来手就在机上，不必先滑一下。
- **先滑后输，键盘不是替代品**：`lock_asking` 只在滑过六成之后置位，所以在任何锁屏上
  第一眼看到的都是滑动页；密码设置与否只决定滑完之后进页面还是进键盘。
- **锁屏期间不重画页面**：模糊背景已有缓存时，`compose()` 跳过 `compose_page()`——拉动时
  每帧只画锁屏本身，页面重画一遍纯属浪费（这一条也是模糊背景能用的前提）。
- **不压暗，让字色跟着玻璃走**：整层压暗会把浅色页面压成一片均匀的中灰（第一版就是这样，
  太闷），只压上下两端又会在中间留一道生硬的分界。索性一点不压：上锁时用 `luma()` 读一次
  玻璃的平均亮度，亮玻璃用深墨字、暗玻璃用白字。浅色页面上的锁屏因此是黑字，深色页面上
  仍是白字，两边都不必为字让出一块暗底。
- **字色三选，默认跟随**：跟随系统就是原来那套读玻璃亮度；浅色与深色是把字色定死——
  浅色模式给深字，深色模式给白字。默认跟随：玻璃本来就透出系统的页面，跟着走不会看错；
  那两个选项留给想要字色与页面相反的人。
- **电量画图不画字**：状态栏那块电池把数字塞在壳子里，锁屏上已经有一个大号蓝字百分比，
  再塞一次就重了。所以锁屏的电池只留形状与填充，数字交给旁边那行字，两者同一份蓝色。
- **震动走 `Effect::Haptic`**，与页面里的长按、截图同一个出口，`state.vibrate` 关掉就静音。
- **按键也走锁屏**：除电源键外，锁屏期间任何按键都不透传到页面（否则 `返回` 会翻页）。
- **充电屏画在锁屏之上**：插线先看到充电动画，划走才落到锁屏。
- **`App::new(Screen::SetCode)` 也建 `code_entry`**：构造一个以设密码页开场的
  `App`（预览与测试会这么做）时，它没经过 `open()`，入口是 `None` 的话键盘输入会静默丢弃。

## 三、版本号（`crates/skyward-info`）

```rust
pub const UPSTREAM: &str = env!("CARGO_PKG_VERSION");   // 0.2.0，更新检查用这个
pub const PORT: &str = "0.2.2";                         // 本移植的第几版
pub const VERSION: &str = concat!("SW-v", env!("CARGO_PKG_VERSION"), "-nuwa-v0.2.2");
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
