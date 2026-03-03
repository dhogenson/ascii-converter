use std::io::{self, Write};

pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    message: String,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            width: 40,
            message: String::new(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn increment(&mut self) {
        if self.current < self.total {
            self.current += 1;
            self.draw();
        }
    }

    fn draw(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as usize
        } else {
            0
        };

        let filled = if self.total > 0 {
            (self.current as f64 / self.total as f64 * self.width as f64) as usize
        } else {
            0
        };

        let empty = self.width.saturating_sub(filled);

        let bar: String = std::iter::repeat('=')
            .take(filled)
            .chain(std::iter::repeat('-').take(empty))
            .collect();

        let msg = if self.message.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.message)
        };

        print!(
            "\r[{bar}] {percentage:>3}%{msg}",
            bar = bar,
            percentage = percentage,
            msg = msg
        );
        io::stdout().flush().unwrap();
    }

    pub fn finish(&self) {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_new() {
        let pb = ProgressBar::new(100);
        assert_eq!(pb.total, 100);
        assert_eq!(pb.current, 0);
    }

    #[test]
    fn test_progress_bar_increment() {
        let mut pb = ProgressBar::new(100);
        pb.increment();
        assert_eq!(pb.current, 1);
    }

    #[test]
    fn test_progress_bar_with_message() {
        let pb = ProgressBar::new(100).with_message("Processing");
        assert_eq!(pb.message, "Processing");
    }
}
