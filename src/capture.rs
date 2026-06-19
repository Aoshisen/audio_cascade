use screencapturekit::cm::{ CMSampleBuffer, CMSampleBufferExt };
use screencapturekit::prelude::*;
use screencapturekit::stream::output_type::SCStreamOutputType;
use std::sync::mpsc;
use std::sync::Arc;

pub struct AudioCapture {
    pub sample_rate: u32,
    rx: mpsc::Receiver<Vec<f32>>,
    _stream: SCStream,
}

impl AudioCapture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 1. 获取可共享内容（需要屏幕录制权限）
        let content = SCShareableContent::get().map_err(|e| {
            format!("\
                 无法获取屏幕内容：{}\n\
                 请在 系统设置 → 隐私与安全性 → 屏幕录制 中授权，\n\
                 然后重新运行程序。", e)
        })?;

        let display = content
            .displays()
            .into_iter()
            .next()
            .ok_or_else(|| "未找到显示器".to_string())?;

        println!("显示器: {}x{}", display.width(), display.height());

        // 2. 构建内容过滤器（SCK 即使只采集音频也需要一个 display filter）
        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build();

        // 3. 构建流配置
        let sample_rate = 48000u32;
        let channel_count = 2;
        let config = SCStreamConfiguration::new()
            .with_captures_audio(true)
            .with_sample_rate(sample_rate as i32)
            .with_channel_count(channel_count)
            .with_excludes_current_process_audio(true);

        println!("采样率: {} Hz, 通道数: {}", sample_rate, channel_count);

        // 4. 创建音频数据通道
        let (tx, rx) = mpsc::channel();
        let tx = Arc::new(tx);

        // 5. 创建流并注册音频处理器
        let mut stream = SCStream::new(&filter, &config);
        let handler_tx = tx.clone();

        stream.add_output_handler(move |sample: CMSampleBuffer, of_type: SCStreamOutputType| {
            if of_type != SCStreamOutputType::Audio {
                return;
            }

            if let Some(audio_list) = sample.audio_buffer_list() {
                for audio_buf in audio_list.iter() {
                    let channels = audio_buf.number_channels;
                    let data = audio_buf.data();

                    if data.is_empty() || data.len() % 4 != 0 {
                        continue;
                    }

                    // 将字节重新解释为 f32 PCM 样本
                    let samples: &[f32] = unsafe {
                        std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() / 4)
                    };

                    if channels == 2 {
                        // 立体声 → 单声道下混: (L + R) / 2
                        let mono: Vec<f32> = samples
                            .chunks(2)
                            .map(|ch| (ch[0] + ch[1]) * 0.5)
                            .collect();
                        let _ = handler_tx.send(mono);
                    } else if channels == 1 {
                        // 已经是单声道
                        let _ = handler_tx.send(samples.to_vec());
                    }
                }
            }
        }, SCStreamOutputType::Audio);

        // 6. 开始捕获
        stream
            .start_capture()
            .map_err(|e| {
                format!("\
                 启动音频捕获失败：{}\n\
                 请确保已在 系统设置 → 隐私与安全性 → 屏幕录制 中授权，\n\
                 然后重新运行程序。", e)
            })?;

        println!("音频捕获已启动 (ScreenCaptureKit)");

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
