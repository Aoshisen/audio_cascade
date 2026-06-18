use realfft::{RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use std::sync::Arc;

/// 对音频帧做 FFT 并将结果分组为指定数量的频段
pub struct FrequencyAnalyzer {
    fft_size: usize,
    num_bands: usize,
    r2c: Arc<dyn RealToComplex<f32>>,
    window: Vec<f32>,
    spectrum: Vec<Complex<f32>>,
    /// 各频段的 bin 起始索引（长度 num_bands + 1）
    band_limits: Vec<usize>,
}

impl FrequencyAnalyzer {
    pub fn new(
        fft_size: usize,
        num_bands: usize,
        sample_rate: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(fft_size);

        // Hanning 窗
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos())
            })
            .collect();

        let spectrum = vec![Complex::new(0.0, 0.0); fft_size / 2 + 1];

        // 对数频段分组：从 30Hz 到 Nyquist，跳过 DC 分量
        let nyquist_bins = fft_size / 2;
        let nyquist_hz = sample_rate as f32 / 2.0;
        let min_freq = 30.0f32;

        let mut band_limits: Vec<usize> = Vec::with_capacity(num_bands + 1);

        for i in 0..=num_bands {
            let ratio = i as f32 / num_bands as f32;
            let freq_hz = min_freq * (nyquist_hz / min_freq).powf(ratio);
            let bin = (freq_hz / nyquist_hz * nyquist_bins as f32) as usize;
            band_limits.push(bin.min(nyquist_bins));
        }
        // 去重，确保每个频段至少包含 1 个 bin
        for i in 1..band_limits.len() {
            if band_limits[i] <= band_limits[i - 1] {
                band_limits[i] = band_limits[i - 1] + 1;
            }
        }
        band_limits[num_bands] = band_limits[num_bands].min(nyquist_bins);

        Ok(Self {
            fft_size,
            num_bands,
            r2c,
            window,
            spectrum,
            band_limits,
        })
    }

    /// 对一帧音频做 FFT，返回各频段的归一化幅度 (0.0 ~ 1.0)
    pub fn analyze(&mut self, samples: &[f32]) -> Vec<f32> {
        assert_eq!(samples.len(), self.fft_size);

        // 去除直流偏置
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;

        // 加窗（在去除 DC 之后）
        let mut input: Vec<f32> = samples
            .iter()
            .zip(self.window.iter())
            .map(|(s, w)| (s - mean) * w)
            .collect();

        // FFT
        self.r2c
            .process(&mut input, &mut self.spectrum)
            .expect("FFT 处理失败");

        // 计算各 bin 的幅度 (只取前一半，即正频率)
        let nyquist = self.fft_size / 2;
        let bin_mags: Vec<f32> = self.spectrum[..nyquist]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        // 对数频段分组
        let mut result = vec![0.0f32; self.num_bands];
        for i in 0..self.num_bands {
            let start = self.band_limits[i];
            let end = self.band_limits[i + 1];
            if end > start {
                let sum: f32 = bin_mags[start..end].iter().sum();
                result[i] = sum / (end - start) as f32;
            }
        }

        // 归一化
        let max_val = result.iter().cloned().fold(0.0f32, f32::max);
        if max_val > 0.0 {
            for v in &mut result {
                *v /= max_val;
            }
        }

        result
    }
}
