use std::collections::VecDeque;

pub struct FpsCounter {
    timestamps: VecDeque<f64>,
    window_duration: f64,
}

impl FpsCounter {
    pub fn new(window_seconds: f32) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_duration: window_seconds as f64,
        }
    }

    pub fn update(&mut self, current_time: f64) {
        self.timestamps.push_back(current_time);

        while let Some(&oldest) = self.timestamps.front() {
            if current_time - oldest > self.window_duration {
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

        let duration = self.timestamps.back().unwrap() - self.timestamps.front().unwrap();
        if duration <= 0.0 {
            return 0.0;
        }

        ((self.timestamps.len() - 1) as f64 / duration) as f32
    }
}
