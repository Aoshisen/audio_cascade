mod analyzer;
mod capture;
mod renderer;

use analyzer::FrequencyAnalyzer;
use capture::AudioCapture;
use renderer::SpectrumRenderer;

use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const FFT_SIZE: usize = 4096;
const HOP_SIZE: usize = FFT_SIZE / 2; // 50% 重叠
const NUM_BANDS: usize = 50;

const WIN_W: usize = 800;
const WIN_H: usize = 400;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let capture = AudioCapture::new()?;
    let sample_rate = capture.sample_rate;
    let mut analyzer = FrequencyAnalyzer::new(FFT_SIZE, NUM_BANDS, sample_rate)?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Audio Spectrum")
        .with_inner_size(winit::dpi::LogicalSize::new(WIN_W as f64, WIN_H as f64))
        .build(&event_loop)?;

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(WIN_W as u32, WIN_H as u32, &window);
        Pixels::new(WIN_W as u32, WIN_H as u32, surface_texture)?
    };

    let mut renderer = SpectrumRenderer::new(NUM_BANDS, WIN_W, WIN_H);

    let mut buf: Vec<f32> = Vec::with_capacity(FFT_SIZE * 4);
    let mut last_spectrum = vec![0.0f32; NUM_BANDS];

    println!("按 Ctrl+C 或关闭窗口退出");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let _ = pixels.resize_surface(size.width, size.height);
            }
            Event::RedrawRequested(_) => {
                // 处理音频
                if let Some(samples) = capture.next_samples() {
                    buf.extend(samples);
                    while buf.len() >= FFT_SIZE {
                        let frame: Vec<f32> = buf[..FFT_SIZE].to_vec();
                        buf.drain(..HOP_SIZE);
                        last_spectrum = analyzer.analyze(&frame);
                    }
                }

                // 渲染
                renderer.render(pixels.frame_mut(), &last_spectrum);
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
