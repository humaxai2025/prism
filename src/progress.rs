use std::time::{Duration, Instant};
use std::io::{self, Write};

pub struct ProgressIndicator {
    enabled: bool,
    start_time: Instant,
    total_items: Option<usize>,
    current_item: usize,
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressIndicator {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            start_time: Instant::now(),
            total_items: None,
            current_item: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }

    pub fn with_total(enabled: bool, total: usize) -> Self {
        Self {
            enabled,
            start_time: Instant::now(),
            total_items: Some(total),
            current_item: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }

    pub fn set_total(&mut self, total: usize) {
        self.total_items = Some(total);
    }

    pub fn increment(&mut self, message: &str) {
        if !self.enabled {
            return;
        }

        self.current_item += 1;
        let now = Instant::now();

        if now.duration_since(self.last_update) >= self.update_interval {
            self.update_display(message);
            self.last_update = now;
        }
    }

    pub fn set_message(&self, message: &str) {
        if !self.enabled {
            return;
        }
        print!("\r🔍 {}", message);
        io::stdout().flush().unwrap_or(());
    }

    pub fn finish(&self, message: &str) {
        if !self.enabled {
            return;
        }

        let elapsed = self.start_time.elapsed();
        print!("\r✅ {} ", message);
        
        if let Some(total) = self.total_items {
            print!("({}/{} items) ", self.current_item, total);
        }
        
        print!("completed in {:.1}s\n", elapsed.as_secs_f64());
        io::stdout().flush().unwrap_or(());
    }

    pub fn error(&self, message: &str) {
        if !self.enabled {
            return;
        }

        print!("\r❌ {} \n", message);
        io::stdout().flush().unwrap_or(());
    }

    fn update_display(&self, message: &str) {
        let elapsed = self.start_time.elapsed();
        
        if let Some(total) = self.total_items {
            let percentage = if total > 0 {
                (self.current_item as f64 / total as f64 * 100.0).round() as u32
            } else {
                0
            };

            let bar_width = 20;
            let filled = (percentage as f64 / 100.0 * bar_width as f64) as usize;
            let empty = bar_width - filled;
            
            let bar = format!("{}{}",
                "█".repeat(filled),
                "░".repeat(empty)
            );

            // Calculate ETA
            let eta_str = if self.current_item > 0 && elapsed.as_secs() > 0 {
                let rate = self.current_item as f64 / elapsed.as_secs_f64();
                let remaining = total - self.current_item;
                let eta_secs = remaining as f64 / rate;
                
                if eta_secs < 60.0 {
                    format!("ETA: {:.0}s", eta_secs)
                } else {
                    format!("ETA: {:.1}m", eta_secs / 60.0)
                }
            } else {
                "ETA: --".to_string()
            };

            print!("\r🔍 {} [{}] {}% ({}/{}) {} ",
                message, bar, percentage, self.current_item, total, eta_str);
        } else {
            // Indeterminate progress
            let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let spinner_index = (elapsed.as_millis() / 80) as usize % spinner_chars.len();
            let spinner = spinner_chars[spinner_index];
            
            print!("\r{} {} ({} processed, {:.1}s) ",
                spinner, message, self.current_item, elapsed.as_secs_f64());
        }
        
        io::stdout().flush().unwrap_or(());
    }

    pub fn step(&mut self, message: &str) -> ProgressStep {
        ProgressStep::new(self, message)
    }
}

pub struct ProgressStep<'a> {
    progress: &'a mut ProgressIndicator,
    message: String,
}

impl<'a> ProgressStep<'a> {
    fn new(progress: &'a mut ProgressIndicator, message: &str) -> Self {
        progress.set_message(message);
        Self {
            progress,
            message: message.to_string(),
        }
    }

    pub fn complete(self) {
        self.progress.increment(&self.message);
    }

    pub fn error(self, error_msg: &str) {
        self.progress.error(&format!("{}: {}", self.message, error_msg));
    }
}

// Spinner for simple operations
pub struct Spinner {
    enabled: bool,
    message: String,
    start_time: Instant,
}

impl Spinner {
    pub fn new(enabled: bool, message: &str) -> Self {
        let spinner = Self {
            enabled,
            message: message.to_string(),
            start_time: Instant::now(),
        };
        
        if enabled {
            print!("🔍 {}...", message);
            io::stdout().flush().unwrap_or(());
        }
        
        spinner
    }

    pub fn update(&self, new_message: &str) {
        if self.enabled {
            print!("\r🔍 {}...", new_message);
            io::stdout().flush().unwrap_or(());
        }
    }

    pub fn succeed(self, message: Option<&str>) {
        if self.enabled {
            let elapsed = self.start_time.elapsed();
            let msg = message.unwrap_or(&self.message);
            print!("\r✅ {} completed in {:.1}s\n", msg, elapsed.as_secs_f64());
            io::stdout().flush().unwrap_or(());
        }
    }

    pub fn fail(self, message: Option<&str>) {
        if self.enabled {
            let msg = message.unwrap_or(&self.message);
            print!("\r❌ {} failed\n", msg);
            io::stdout().flush().unwrap_or(());
        }
    }
}

// Helper for parallel processing progress
pub struct ParallelProgress {
    enabled: bool,
    total_files: usize,
    completed: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    start_time: Instant,
}

impl ParallelProgress {
    pub fn new(enabled: bool, total_files: usize) -> Self {
        Self {
            enabled,
            total_files,
            completed: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn increment(&self) -> usize {
        let completed = self.completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        
        if self.enabled {
            let elapsed = self.start_time.elapsed();
            let percentage = (completed as f64 / self.total_files as f64 * 100.0) as u32;
            
            print!("\r🔍 Processing files... {}/{} ({}%) - {:.1}s",
                completed, self.total_files, percentage, elapsed.as_secs_f64());
            io::stdout().flush().unwrap_or(());
        }
        
        completed
    }

    pub fn finish(&self) {
        if self.enabled {
            let elapsed = self.start_time.elapsed();
            println!("\r✅ Completed processing {} files in {:.1}s", 
                self.total_files, elapsed.as_secs_f64());
        }
    }
}