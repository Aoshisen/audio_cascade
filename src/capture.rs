use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, StreamConfig};
use std::sync::mpsc;

pub struct AudioCapture {
    pub sample_rate: u32,
    rx: mpsc::Receiver<Vec<f32>>,
    _stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        let device = host
            .input_devices()?
            .find(|d| d.name().map(|n| n.contains("BlackHole")).unwrap_or(false))
            .ok_or_else(|| {
                "\
                  BlackHole 2ch 未找到！
                  安装: brew install blackhole-2ch
                然后重启电脑。
                使用前还需要在「音频 MIDI 设置」中：
                1. 点击 + → 创建多输出设备
                2. 勾选你的耳机/音箱 + BlackHole 2ch
                3. 系统声音设置中选中这个多输出设备"
                    .to_string()
            })?;

        println!("设备: {}", device.name()?);

        let supported = device.default_input_config()?;
        let sample_rate: u32 = supported.sample_rate().0;
        let channels = supported.channels();
        let config: StreamConfig = supported.into();

        println!("采样率: {} Hz, 通道数: {}", sample_rate, channels);

        let (tx, rx) = mpsc::channel();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &InputCallbackInfo| {
                if channels == 2 {
                    let mono: Vec<f32> = data.chunks(2).map(|ch| (ch[0] + ch[1]) * 0.5).collect();
                    let _ = tx.send(mono);
                } else {
                    let _ = tx.send(data.to_vec());
                }
            },
            move |err| eprintln!("捕获错误: {}", err),
            None,
        )?;

        stream.play()?;
        println!("音频捕获已启动");

        Ok(Self {
            sample_rate,
            rx,
            _stream: stream,
        })
    }

    /// 读取当前积压的音频样本（非阻塞）
    pub fn next_samples(&self) -> Option<Vec<f32>> {
        let mut samples = Vec::new();
        while let Ok(data) = self.rx.try_recv() {
            samples.extend(data);
        }
        if samples.is_empty() {
            None
        } else {
            Some(samples)
        }
    }
}
