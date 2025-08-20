use std::time::Instant;

pub struct Counter {
    max: u64,
    count: u64,
    duration: Option<Instant>
}

impl Counter {
    pub fn new(max :u64) -> Self {
        Self { 
            max,
            count: 0,
            duration: None
        }
    }

    pub fn limit(&mut self) -> bool {
        if let Some(d) = self.duration {
            let time = d.elapsed();
            if 1 < time.as_secs() {
                self.count = 0;
                self.duration = Some(Instant::now());
            }
        } else {
            self.duration = Some(Instant::now());
        }

        if self.max < self.count {
            return true;
        }
        self.count += 1;
        
        false
    }
}