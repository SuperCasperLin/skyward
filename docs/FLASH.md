# 刷入与回退

## 前置

- Bootloader 已解锁（`fastboot flashing unlock`）
- 电脑有 `adb` 与 `fastboot`
- **先备份原厂 `recovery` 分区**（见下），否则回退只能靠官方 OTA / 线刷包

小米 13 Pro 有独立的 `recovery` 分区，不在 `boot` / `init_boot` 里，所以刷 recovery
不动系统、不影响 `_a`/`_b` 槽位里的数据。

## 备份原厂 recovery

两种方式，任选：

**A. 从官方 OTA 抽**（无需 root）

```bash
# payload.bin 里就有 recovery.img
payload_dumper --out extracted payload.bin
cp extracted/recovery.img stock-recovery-nuwa.img
```

**B. 在已 root 的系统里直接 dd**

```bash
adb shell su -c "dd if=/dev/block/by-name/recovery of=/sdcard/stock-recovery-nuwa.img"
adb pull /sdcard/stock-recovery-nuwa.img
```

拿到后记下 `sha256sum`，回退时用得上。

## 刷入

镜像文件名与版本号一致，例如 `SW-v0.2.0-nuwa-v0.2.3.img`；先核对同名 `.sha256`：

```bash
sha256sum -c SW-v0.2.0-nuwa-v0.2.3.img.sha256
adb reboot bootloader
fastboot devices
fastboot flash recovery SW-v0.2.0-nuwa-v0.2.3.img
fastboot reboot recovery
```

`fastboot flash` 之后不一定自动进 recovery，用 `fastboot reboot recovery` 或
长按电源键选「重启到 Recovery」。

进入后首次需要**输入锁屏密码解锁 data**——这正是这个移植要解决的问题。

## 为什么不建议 `fastboot boot`

部分机型的 bootloader 会忽略 `fastboot boot <img>`（返回 OKAY 但什么都没发生）。
本机型建议直接 `flash recovery`，反正是独立分区，风险很低。

## 回退

```bash
adb reboot bootloader
fastboot flash recovery stock-recovery-nuwa.img
fastboot reboot recovery
```

回退后系统、data、decrypted 状态都不受影响。若想彻底恢复官方状态，也可以直接刷官方
线刷包（注意会清数据）。

## 常见问题

**刷完卡在开机 logo？** 说明底座和本机固件版本不匹配。本项目使用的底座来自
`OS3.0.312.0.WMBCNXM`，不同版本的 `recovery.img` 结构（尤其 ramdisk 压缩方式）可能不同。

**能进界面但 `/data` 挂不上？** 检查 `devices/nuwa.toml` 里的 5 个 TEE 服务是否都在
（`qseecomd`、`keymint-service-qti`、`hwservicemanager`、`gatekeeper@1.0-service-qti`、
`boot@1.2-service`），以及打包时是否删掉了 `ro.build.version.release` 与两条
security patch 属性——KeyMint 要靠这些向 TEE 报构建版本。

**密码一直报错？** 报 `the secure world does not recognise that credential` 说明链路通了、
只是密码不对（或该用户没设密码）。报其它错误才是链路问题。

## 调试

刷入后 `adb` 在 recovery 里是可用的（沿用原厂 adbd）：

```bash
adb shell dmesg | grep -i skyward     # 启动与解密过程
adb shell cat /metadata/skyward/settings.json   # 设置
adb shell cat /sys/class/backlight/panel0-backlight/brightness  # 背光（AOD 时应为约 2% 量程）
```
