use crate::disco::Disco;
use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use std::time::{Instant, Duration};

#[test]
fn test_screensaver_performance() {
    let mut disco = Disco::new();
    // Prevent slow system info calls by setting sys_refresh_timer way below 0
    disco.sys_refresh_timer = -1000.0;
    
    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    let dt = Duration::from_millis(16);
    
    let start = Instant::now();
    for _ in 0..100 {
        disco.update(dt, cols, rows);
        disco.draw(&mut grid, cols, rows);
    }
    let duration = start.elapsed();
    
    println!("Performance test took: {:?}", duration);
    // Assert completing 100 frames finishes within 1500ms budget.
    assert!(duration < Duration::from_millis(1500), "Performance test exceeded 1500ms budget: {:?}", duration);
}
