use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct FpsCounter {
    timestamps: VecDeque<Instant>,
    window_duration: Duration,
}

impl FpsCounter {
    pub fn new(window_seconds: f32) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_duration: Duration::from_secs_f32(window_seconds),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.timestamps.push_back(now);

        while let Some(&oldest_timestamp) = self.timestamps.front() {
            if now.duration_since(oldest_timestamp) > self.window_duration {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn fps(&self) -> f32 {
        if self.timestamps.len() < 2 {
            return 0.0;
        }

        let oldest_timestamp = self.timestamps.front().unwrap();
        let newest_timestamp = self.timestamps.back().unwrap();

        let duration = newest_timestamp.duration_since(*oldest_timestamp);

        if duration.as_secs_f32() == 0.0 {
            return 0.0;
        }

        let num_frames = (self.timestamps.len() - 1) as f32;

        num_frames / duration.as_secs_f32()
    }
}
