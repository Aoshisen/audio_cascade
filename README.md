# Audio Cascade

基于 ScreenCaptureKit 的系统音频频谱可视化工具。

## 原理

利用 Apple 的 ScreenCaptureKit 框架捕获系统音频输出，通过 FFT 分析实时频谱，并在窗口中可视化。

## 依赖

- **screencapturekit** — 捕获系统音频（替代旧版的 BlackHole + cpal）
- **realfft / rustfft** — 实时 FFT 频谱分析
- **winit + pixels** — 窗口与像素渲染
- **macOS 13+** — ScreenCaptureKit 需要此版本

## 权限

首次运行需要在 **系统设置 → 隐私与安全性 → 屏幕录制** 中授权，然后重新运行程序。

无需安装 BlackHole 或配置「音频 MIDI 设置」中的多输出设备。

## 使用

```
cargo run
```

按 Ctrl+C 或关闭窗口退出。
