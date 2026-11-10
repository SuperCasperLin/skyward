# 构建

在 Windows（Git Bash）上验证过；Linux/macOS 上把路径转换相关的几行去掉即可。

## 依赖

| 依赖 | 版本（验证过的） | 说明 |
|---|---|---|
| Rust | 1.99.1 `stable-x86_64-pc-windows-gnu` | 需要 `aarch64-linux-android` target |
| Android NDK | r29 | 解压到 `tools/android-ndk-r29` |
| cargo-ndk | 4.1.2 | `cargo install cargo-ndk` |
| lz4 CLI | v1.10.0 win64 | 解压到 `tools/lz4`，打包器要用它做 legacy 帧封装 |

```bash
rustup target add aarch64-linux-android
cargo install cargo-ndk
# NDK：https://developer.android.com/ndk/downloads  → tools/android-ndk-r29
# lz4：https://github.com/lz4/lz4/releases            → tools/lz4/lz4.exe
```

`skyward-pack` 的 `Ramdisk::encode` 会调用外部 `lz4`，且必须产出 **legacy 帧格式**，
所以不能用纯 Rust 的 lz4 实现替代。

## 一次完整构建

```bash
bash build.sh
```

脚本按顺序做三件事：

1. **二进制**（`cargo ndk -t arm64-v8a --platform 30`）：`skyward`（设备端 UI，静态链接
   system-init 之外的部分）、`skyward-pack`（宿主端打包器）、`system-init`（`+crt-static`
   静态链接，单独 target-dir）。
2. **原厂底座**：`skyward-pack stock` 从官方 OTA 抽出的 `recovery.img` 取底座，缓存到
   `upstream/target/stock/nuwa/recovery.img`，之后复用。
3. **打包**：把 `skyward` 二进制、`system-init`、`devices/nuwa.toml`、字体塞进 ramdisk，
   替换掉 `system/bin/recovery`，重新压成 100 MiB 的镜像。

产物：

```
upstream/target/out/SW-v0.2.0-nuwa-v0.2.0/
├── skyward-nuwa.img
└── skyward-nuwa.img.sha256
```

镜像名里的版本来自仓库根的 `VERSION` 文件。

## 分步构建

```bash
bash build.sh binaries   # 只编二进制
bash build.sh pack       # 只重新打包（改配置时用这个，几十秒）
```

改 `devices/nuwa.toml` 之后**必须重新打包**——配置是打进 ramdisk 的，不是运行时读外部的。

## 踩过的坑

**Git Bash 会改写路径参数。** 脚本开头已经设了：

```bash
export MSYS_NO_PATHCONV=1
export MSYS2_ARG_CONV_EXCL='*'
```

手工调用 `adb pull /sdcard/x` 之类时也要设，否则 `C:/Program Files/...` 会被替换成 `/c/...`。

**cargo-ndk 只认 Windows 风格路径。** `ANDROID_NDK_HOME` 必须是 `C:/...` 而不是 `/c/...`，
脚本里用 `cygpath -m` 转换。传给 `adb`/`fastboot` 的镜像路径则用 `cygpath -w`。

**宿主编译不通是正常的。** `cargo check -p skyward-ui` 在 Windows 上会失败——依赖链里的
`skyward-lp`/`rsproperties` 只支持 Unix。判断代码是否正确请用交叉检查：

```bash
export ANDROID_NDK_HOME="$(cygpath -m "$(pwd)/tools/android-ndk-r29")"
cd upstream && cargo ndk -t arm64-v8a --platform 30 check -p skyward
```

**跑测试。** 宿主机跑不了 `cargo test`（同上），做法是交叉编译测试二进制推到手机上跑：

```bash
cargo ndk -t arm64-v8a --platform 30 test --no-run -p skyward-ui
adb push upstream/target/aarch64-linux-android/debug/deps/skyward_ui-<hash> /tmp/ui-tests
adb push upstream/resources/fonts/MiSansVF.ttf /tmp/fonts/MiSansVF.ttf
adb push upstream/resources/fonts/KirkMono-Regular.ttf /tmp/fonts/KirkMono-Regular.ttf
adb shell "chmod 755 /tmp/ui-tests; SKYWARD_FONT_DIR=/tmp/fonts /tmp/ui-tests --test-threads=1"
```

`SKYWARD_FONT_DIR` 让测试从设备上的目录取字体（`font::bundled()` 支持这个环境变量）。
`lang::tests::the_typeface_draws_every_text_the_pages_write` 需要读取源码树，在手机上无法运行，
属预期失败。

## 预览页面

`skyward-preview` 可以在不刷机的情况下把页面渲染成 PNG（同样交叉编译到设备上跑，用真字体
和真分辨率）：

```bash
cargo ndk -t arm64-v8a --platform 30 build --release -p skyward-preview
adb push upstream/target/aarch64-linux-android/release/skyward-preview /tmp/
adb push devices/nuwa.toml /tmp/nuwa.toml

# 整套页面
adb shell "cd /tmp && SKYWARD_FONT=/tmp/MiSansVF.ttf ./skyward-preview /tmp/pages --config /tmp/nuwa.toml --lang zh-Hans"

# 只出息屏显示
adb shell "cd /tmp && ./skyward-preview /tmp/aod --aod --config /tmp/nuwa.toml --font /tmp/MiSansVF.ttf"
```
