pub struct SpectrumRenderer {
    num_bands: usize,
    width: usize,
    height: usize,
    smooth_data: Vec<f32>,
    smoothing: f32,
    gravity: f32,
    cap_y: Vec<f32>,
    cap_vy: Vec<f32>,
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

    pub fn render(&mut self, frame: &mut [u8], magnitudes: &[f32]) {
        if !magnitudes.is_empty() {
            for (i, &m) in magnitudes.iter().enumerate() {
                self.smooth_data[i] =
                    self.smooth_data[i] * (1.0 - self.smoothing) + m * self.smoothing;
            }
        }

        let w = self.width;
        let h = self.height;

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

        for i in 0..self.num_bands {
            let val = self.smooth_data[i];
            let bar_h = (val * graph_h as f32) as usize;
            let x = graph_left + gap + i * (bar_w + gap);
            let bar_y = graph_bottom - bar_h;

            draw_bar(frame, w, x, bar_y, bar_w, bar_h);

            // 帽子物理
            let bar_top = graph_bottom as f32 - bar_h as f32;
            let cap_top = self.update_cap(i, bar_top, graph_bottom);
            draw_cap(frame, w, x, cap_top, bar_w, graph_bottom);
        }

        draw_baseline(frame, w, margin, w - margin, h - bottom_margin);
    }

    fn update_cap(&mut self, i: usize, bar_top: f32, graph_bottom: usize) -> usize {
        let cap = &mut self.cap_y[i];
        let vel = &mut self.cap_vy[i];

        if *cap == 0.0 {
            *cap = bar_top;
        }
        if bar_top < *cap {
            *cap = bar_top;
            *vel = 0.0;
        } else {
            *vel += self.gravity;
            *cap += *vel;
            if *cap >= graph_bottom as f32 {
                *cap = graph_bottom as f32;
                *vel = 0.0;
            }
        }

        (*cap as usize).min(graph_bottom - 1)
    }
}

// --- 基本绘制原语 ---

fn fill_rect(frame: &mut [u8], stride: usize, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
    for row in y..y + h {
        let base = (row * stride + x) * 4;
        for col in 0..w {
            let px = base + col * 4;
            frame[px..px + 4].copy_from_slice(&color);
        }
    }
}

fn draw_bar(frame: &mut [u8], stride: usize, x: usize, y: usize, w: usize, h: usize) {
    fill_rect(frame, stride, x, y, w, h, [102, 255, 102, 255]);
}

fn draw_cap(frame: &mut [u8], stride: usize, x: usize, top: usize, bar_w: usize, graph_bottom: usize) {
    let h = 2.min(graph_bottom - top);
    fill_rect(frame, stride, x, top, bar_w, h, [255, 255, 200, 255]);
}

fn draw_baseline(frame: &mut [u8], stride: usize, x_start: usize, x_end: usize, y: usize) {
    fill_rect(frame, stride, x_start, y, x_end - x_start, 1, [68, 68, 68, 255]);
}
