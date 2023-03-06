use std::mem;
use winapi::shared::ntdef::LARGE_INTEGER;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub struct Time {
    pub game_time: f64,
    pub delta_time: f64,

    m_seconds_per_count: f64,

    m_base_time: i64,
    m_paused_time: i64,
    m_stop_time: i64,
    m_prev_time: i64,
    m_curr_time: i64,

    m_stopped: bool,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        unsafe {
            let mut counts_per_sec: LARGE_INTEGER = mem::zeroed();
            QueryPerformanceFrequency(&mut counts_per_sec);

            let mut curr_time: LARGE_INTEGER = mem::zeroed();
            QueryPerformanceCounter(&mut curr_time);

            Self {
                game_time: 0.0,
                delta_time: 0.0,
                m_seconds_per_count: 1.0 / *counts_per_sec.QuadPart() as f64,
                m_base_time: 0,
                m_paused_time: 0,
                m_stop_time: 0,
                m_prev_time: *curr_time.QuadPart(),
                m_curr_time: 0,
                m_stopped: false,
            }
        }
    }
    pub fn reset(&mut self) {}
    pub fn start(&mut self) {}
    pub fn stop(&mut self) {}
    pub fn tick(&mut self) {
        if self.m_stopped {
            self.delta_time = 0.0;
            return;
        }

        unsafe {
            // Get the time this frame
            let mut curr_time: LARGE_INTEGER = mem::zeroed();
            QueryPerformanceCounter(&mut curr_time);
            self.m_curr_time = *curr_time.QuadPart();
        }

        // Time difference between this frame and the previous.
        self.delta_time = (self.m_curr_time - self.m_prev_time) as f64 * self.m_seconds_per_count;

        // Prepare for next frame.
        self.m_prev_time = self.m_curr_time;
        // Force nonnegative. The DXSDK's CDXUTTimer mentions that if the
        // processor goes into a power save mode or we get shuffled to another
        // processor, then mDeltaTime can be negative.
        if self.delta_time < 0.0 {
            self.delta_time = 0.0;
        }

        self.game_time += self.delta_time;

        // println!("delta time: {}", self.delta_time);
    }
}
