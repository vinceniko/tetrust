// #![feature(checked_duration_since)] // non-panic'ing version for instant delay checking
// #![feature(duration_float)] // used to determine the number of frames given a frame time and total duration of an animation

use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct FrameTimer {
    frames: Vec<Duration>,
    last_update: Instant,
    next: usize,
}


impl FrameTimer {
    fn init_frameless(delay: Duration) -> Self {
        Self {
            frames: Vec::default(),
            last_update: Instant::now() + delay,
            next: 0,
        }
    }

    fn set_frames(mut self, vec: Vec<Duration>) -> Self {
        self.frames = vec;
        return self
    }

    pub fn from_vec(frames: Vec<Duration>, delay: Duration) -> Self {
        Self::init_frameless(delay).set_frames(frames)
    }

    // n frames of equal duration
    pub fn equal_sized(n_frames: usize, duration: Duration, delay: Duration) -> Self {
        let frames = vec![duration; n_frames];
        Self::init_frameless(delay).set_frames(frames)
    }

    // update self.last_update to now
    fn set_update(&mut self) {
        self.last_update = Instant::now();
    }

    // returns the state of the current frame and advances to the next frame if the state was ready
    pub fn state(&mut self) -> FrameState {
        if self.is_done() {
            return FrameState::Done
        }

        let curr_frame = &self.frames[self.next];

        if self.next == 0 {
            if let None = self.last_update.checked_duration_since(Instant::now()) { // for creating a delay before playing the animation. ie. do not play if now is before the initial last_update
                self.next += 1;
                self.set_update();
                return FrameState::Ready
            }
        } else if Instant::now() - self.last_update >= *curr_frame {
            self.next += 1;
            self.set_update();
            
            return FrameState::Ready
        }
        FrameState::Waiting
    }

    // gets the state but does not advance to the next frame
    pub fn get_state(&self) -> FrameState {
        if self.is_done() {
            return FrameState::Done
        }

        let curr_frame = &self.frames[self.next];

        if self.next == 0 {
            if let None = self.last_update.checked_duration_since(Instant::now()) { // for creating a delay before playing the animation. ie. do not play if now is before the initial last_update
                return FrameState::Ready
            }
        } else if Instant::now() - self.last_update >= *curr_frame {            
            return FrameState::Ready
        }
        FrameState::Waiting
    }

    fn is_done(&self) -> bool {
        self.next == self.frames.len()
    }
}

// a frame can be Ready, Waiting, or Done
pub enum FrameState {
    Ready,
    Waiting,
    Done
}

// an animatable type has the animate method which takes the a frame state to change the draweable state of the instance for the next drawing
pub trait Animatable {
    fn animate(&mut self, state: &FrameState);
}