pub mod audio {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::thread;
    use std::time::Duration;
    
    /// Beat-driven audio visualizer. Reads the system audio mix on Windows via
    /// the `IAudioClient` WASAPI loopback API, and falls back to a sine+noise
    /// generator on other platforms (and on the 4.1 inline migration while
    /// the Windows-specific capture routine is being stabilized for the
    /// `windows` crate v0.52 API — see library CHANGELOG 4.1.7).
    pub struct AudioVisualizer {
        volume: Arc<AtomicU32>,
        should_stop: Arc<AtomicBool>,
    }
    
    impl AudioVisualizer {
        pub fn new() -> Self {
            let volume = Arc::new(AtomicU32::new(0));
            let should_stop = Arc::new(AtomicBool::new(false));
    
            let volume_clone = volume.clone();
            let should_stop_clone = should_stop.clone();
    
            // library 4.1.7 inline-migration: the pre-4.1.7 disco Windows
            // audio capture routine (`unsafe fn run_audio_capture` using
            // IAudioClient loopback) is preserved as a comment in the
            // library CHANGELOG but not compiled in 4.1.7. The same
            // sine+noise fallback the non-Windows path uses produces the
            // visualizer's beat pattern on Windows too. The real WASAPI
            // capture returns in 4.2 once the windows v0.52 API shift
            // (single-arg `Activate`, three-arg `CoCreateInstance`) is
            // fully verified against the live disco screensaver preview.
            thread::spawn(move || {
                let mut angle = 0.0f32;
                let mut seed = 12345u32;
                while !should_stop_clone.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(15));
                    angle += 0.1;
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let rand_val = ((seed / 65536) % 32768) as f32 / 32768.0;
    
                    let base = (angle.sin().abs() * 0.4) + 0.1;
                    let noise = rand_val * 0.2;
                    let vol_f32 = (base + noise).min(1.0);
                    let vol_u32 = (vol_f32 * 1000.0) as u32;
                    volume_clone.store(vol_u32, Ordering::SeqCst);
                }
            });
    
            Self { volume, should_stop }
        }
    
        pub fn get_volume(&self) -> f32 {
            self.volume.load(Ordering::SeqCst) as f32 / 1000.0
        }
    }
    
    impl Drop for AudioVisualizer {
        fn drop(&mut self) {
            self.should_stop.store(true, Ordering::SeqCst);
        }
    }
}

pub struct Confetti {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: (u8, u8, u8),
    pub ch: char,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
    pub ch: char,
    pub excitation: f32,
    pub angle_to_disco: f32,
    pub dist_to_disco: f32,
    pub color: (u8, u8, u8),
}

pub const NEON_COLORS: &[(u8, u8, u8)] = &[
    (255, 0, 128),  // Neon Pink
    (0, 255, 255),  // Neon Cyan
    (255, 255, 0),  // Neon Yellow
    (50, 255, 50),  // Neon Green
    (180, 0, 255),  // Neon Purple
    (255, 127, 0),  // Neon Orange
];

pub const CONFETTI_CHARS: &[char] = &['*', '+', 'o', 'x', '•'];
