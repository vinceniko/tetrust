#[derive(Clone, Debug)]
pub struct FrameTimer {
    frames: Vec<f64>,
    delay: f64,
    last_update: f64,
    next: usize,
}


impl FrameTimer {
    fn init_frameless(delay: f64) -> Self {
        Self {
            frames: Vec::default(),
            delay,
            last_update: 0.0,
            next: 0,
        }
    }

    fn set_frames(mut self, vec: Vec<f64>) -> Self {
        self.frames = vec;
        return self
    }

    #[allow(dead_code)]
    pub fn from_vec(frames: Vec<f64>, delay: f64) -> Self {
        Self::init_frameless(delay).set_frames(frames)
    }

    // n frames of equal duration
    pub fn equal_sized(n_frames: usize, duration: f64, delay: f64) -> Self {
        let frames = vec![duration; n_frames];
        Self::init_frameless(delay).set_frames(frames)
    }

    // update self.last_update to now
    fn set_update(&mut self, elapsed: f64) {
        self.last_update += elapsed;
    }

    // returns the state of the current frame and advances to the next frame if the state was ready
    pub fn state(&mut self, elapsed: f64) -> FrameState {
        if self.is_done() {
            return FrameState::Done
        }

        self.set_update(elapsed);

        let curr_frame = &self.frames[self.next];

        if self.next == 0 {
            if self.last_update > self.delay { // for creating a delay before playing the animation. ie. do not play if now is before the initial last_update
                self.next += 1;
                self.last_update = 0.0;
                return FrameState::Ready
            }
        } else if self.last_update >= *curr_frame {
            self.next += 1;
            self.last_update = 0.0;
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
            if self.last_update > self.delay { // for creating a delay before playing the animation. ie. do not play if now is before the initial last_update
                return FrameState::Ready
            }
        } else if self.last_update >= *curr_frame {            
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