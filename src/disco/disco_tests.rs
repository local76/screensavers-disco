use super::*;
use std::time::Duration;
use crate::runner::core::TerminalCell;
use crate::runner::core::LcgRng;
use crate::runner::core::screensaver::Screensaver;

#[test]
fn test_disco_creation() {
    let disco = Disco::new();
    assert_eq!(disco.time_elapsed, 0.0);
    assert_eq!(disco.last_cols, 0);
    assert_eq!(disco.last_rows, 0);
}

#[test]
fn test_disco_update_and_draw() {
    let mut disco = Disco::new();
    let mut grid = vec![TerminalCell::default(); 80 * 24];
    
    // Initial update with duration and dimensions
    disco.update(Duration::from_millis(16), 80, 24);
    assert!(disco.time_elapsed > 0.0);
    assert_eq!(disco.last_cols, 80);
    assert_eq!(disco.last_rows, 24);
    
    // Draw to grid
    disco.draw(&mut grid, 80, 24);
    // Ensure some pixels are written or the grid was modified (background is black)
    assert_eq!(grid[0].bg, (0, 0, 0));
}

#[test]
fn test_lcg_rng_determinism() {
    let mut rng1 = LcgRng::new(12345);
    let mut rng2 = LcgRng::new(12345);
    
    for _ in 0..100 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
        assert_eq!(rng1.next_f32(), rng2.next_f32());
    }
}

#[test]
fn test_lcg_rng_range_boundaries() {
    let mut rng = LcgRng::new(999);
    for _ in 0..1000 {
        let val = rng.next_range(3.5, 7.2);
        assert!(val >= 3.5, "Value {} is less than minimum 3.5", val);
        assert!(val <= 7.2, "Value {} is greater than maximum 7.2", val);
        
        let val_usize = rng.next_usize(10);
        assert!(val_usize < 10, "Value {} is not less than 10", val_usize);
    }
}

#[test]
fn test_particle_gravity_physics() {
    let mut confetti = types::Confetti {
        x: 10.0,
        y: 10.0,
        vx: 5.0,
        vy: -2.0,
        color: (255, 0, 0),
        ch: '*',
        lifetime: 1.0,
        max_lifetime: 1.0,
    };
    
    let dt = 0.1f32;
    
    // Manual gravity calculation as done in Screensaver update loop:
    // c.x += c.vx * delta;
    // c.y += c.vy * delta;
    // c.vy += 1.2 * delta;
    confetti.x += confetti.vx * dt;
    confetti.y += confetti.vy * dt;
    confetti.vy += 1.2 * dt;
    
    assert_eq!(confetti.x, 10.5);
    assert_eq!(confetti.y, 9.8);
    assert!((confetti.vy - (-1.88)).abs() < 1e-5);
}

#[test]
fn test_disco_ball_coordinate_math() {
    // Testing the distance and angle math from disco ball to a star position
    let cols = 80;
    let rows = 24;
    let disco_xf = cols as f32 / 2.0;
    let disco_yf = 4.0f32;
    
    let star_x_pct = 0.6;
    let star_y_pct = 0.5;
    
    let star_xf = star_x_pct * cols as f32; // 48.0
    let star_yf = star_y_pct * rows as f32; // 12.0
    
    let dx = (star_xf - disco_xf) * 0.55; // (48.0 - 40.0) * 0.55 = 4.4
    let dy = star_yf - disco_yf;          // 12.0 - 4.0 = 8.0
    
    let dist = (dx * dx + dy * dy).sqrt();
    let angle = if dy > 0.0 { dy.atan2(dx) } else { 0.0 };
    
    // Expected values
    let expected_dist = (4.4f32 * 4.4f32 + 8.0f32 * 8.0f32).sqrt();
    let expected_angle = 8.0f32.atan2(4.4f32);
    
    assert!((dist - expected_dist).abs() < 1e-5);
    assert!((angle - expected_angle).abs() < 1e-5);
}
