pub struct SpectrumRenderer {
    num_bands: usize,
    width: usize,
    height: usize,
    smooth_data: Vec<f32>,
    smoothing: f32,
    gravity: f32,     // 自由落体加速度 (像素/帧²)
    cap_y: Vec<f32>,  // 每个 band 帽子当前 y 坐标 (像素, 相对于窗口顶部)
    cap_vy: Vec<f32>, // 每个 band 帽子当前速度 (像素/帧)
}

impl SpectrumRenderer {
    pub fn new(num_bands: usize, width: usize, height: usize) -> Self {
        Self {
            num_bands,
            width,
            height,
            smooth_data: vec![0.0; num_bands],
            smoothing: 0.3,
            gravity: 0.1,
            cap_y: vec![0.0; num_bands],
            cap_vy: vec![0.0; num_bands],
        }
    }

    /// 绘制一帧频谱到 RGBA framebuffer
    pub fn render(&mut self, frame: &mut [u8], magnitudes: &[f32]) {
        if !magnitudes.is_empty() {
            for (i, &m) in magnitudes.iter().enumerate() {
                self.smooth_data[i] =
                    self.smooth_data[i] * (1.0 - self.smoothing) + m * self.smoothing;
            }
        }

        let w = self.width;
        let h = self.height;

        // 黑色背景
        frame.fill(0);

        let margin = 30;
        let gap = 4;
        let bottom_margin = 10;

        let graph_left = margin;
        let graph_right = w - margin;
        let graph_top = 10;
        let graph_bottom = h - bottom_margin;
        let graph_w = graph_right - graph_left;
        let graph_h = graph_bottom - graph_top;

        let bar_w = (graph_w - gap * (self.num_bands + 1)) / self.num_bands;
        let bar_w = bar_w.max(2);

        // 绘制柱形图
        for i in 0..self.num_bands {
            let val = self.smooth_data[i];
            let bar_h = (val * graph_h as f32) as usize;
            let x = graph_left + gap + i * (bar_w + gap);
            let y = graph_bottom - bar_h;

            for row in y..graph_bottom {
                let base = (row * w + x) * 4;
                for col in 0..bar_w {
                    let px = base + col * 4;
                    frame[px] = 102; // R
                    frame[px + 1] = 255; // G
                    frame[px + 2] = 102; // B
                    frame[px + 3] = 255; // A
                }
            }
            // 帽子物理：柱变长时被顶上去，变短时自由落体
            let bar_top = graph_bottom as f32 - bar_h as f32;
            let cap = &mut self.cap_y[i];
            let vel = &mut self.cap_vy[i];

            if *cap == 0.0 {
                // 首次初始化
                *cap = bar_top;
            }
            if bar_top < *cap {
                // 柱变长了 → 帽子被顶上去
                *cap = bar_top;
                *vel = 0.0;
            } else {
                // 柱变短了 → 帽子自由落体
                *vel += self.gravity;
                *cap += *vel;
                if *cap >= graph_bottom as f32 {
                    *cap = graph_bottom as f32;
                    *vel = 0.0;
                }
            }
            // 绘制帽子（亮色横线，2px 高）
            let cap_color = [255u8, 255, 200, 255];
            let cap_top = (*cap as usize).min(graph_bottom - 1);
            for row in cap_top..(cap_top + 2).min(graph_bottom) {
                let base = (row * w + x) * 4;
                for col in 0..bar_w {
                    let px = base + col * 4;
                    frame[px..px + 4].copy_from_slice(&cap_color);
                }
            }
        }

        // 底部灰色基线
        for col in (margin)..(w - margin) {
            let px = ((h - bottom_margin) * w + col) * 4;
            frame[px] = 68;
            frame[px + 1] = 68;
            frame[px + 2] = 68;
            frame[px + 3] = 255;
        }
    }
}
