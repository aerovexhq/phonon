#![deny(unsafe_code)]

//! Circular FIFO ring buffer for high-throughput zero-allocation audio sample streaming.
//!
//! Provides lock-free single-threaded ring buffering with explicit underrun tracking,
//! pre-allocated sample memory, and safe bounds-checked indexing.

/// Circular FIFO audio ring buffer with underrun detection.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioRingBuffer<T = f32> {
    buffer: Vec<T>,
    read_idx: usize,
    write_idx: usize,
    count: usize,
    capacity: usize,
    underruns: u64,
}

impl<T: Copy + Default> AudioRingBuffer<T> {
    /// Creates a new audio ring buffer with pre-allocated storage capacity.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            buffer: vec![T::default(); cap],
            read_idx: 0,
            write_idx: 0,
            count: 0,
            capacity: cap,
            underruns: 0,
        }
    }

    /// Pushes a single audio sample into the FIFO buffer.
    ///
    /// Returns true if the sample was successfully stored, or false if the buffer was full.
    /// Involves strictly zero heap allocations.
    #[inline(always)]
    pub fn push(&mut self, sample: T) -> bool {
        if self.count >= self.capacity {
            return false;
        }
        self.buffer[self.write_idx] = sample;
        self.write_idx = (self.write_idx + 1) % self.capacity;
        self.count += 1;
        true
    }

    /// Pushes a single audio sample, overwriting the oldest stored sample if full.
    /// Involves strictly zero heap allocations.
    #[inline(always)]
    pub fn push_overwrite(&mut self, sample: T) {
        if self.count >= self.capacity {
            self.read_idx = (self.read_idx + 1) % self.capacity;
        } else {
            self.count += 1;
        }
        self.buffer[self.write_idx] = sample;
        self.write_idx = (self.write_idx + 1) % self.capacity;
    }

    /// Pulls a single sample from the front of the FIFO buffer.
    ///
    /// If the buffer is empty, returns silence (T::default(), e.g. 0.0)
    /// and increments the internal underrun counter.
    /// Involves strictly zero heap allocations.
    #[inline(always)]
    pub fn pull(&mut self) -> T {
        if self.count == 0 {
            self.underruns += 1;
            return T::default();
        }
        let sample = self.buffer[self.read_idx];
        self.read_idx = (self.read_idx + 1) % self.capacity;
        self.count -= 1;
        sample
    }

    /// Peeks at the next available sample at the read pointer without consuming it.
    ///
    /// Returns None if the buffer is empty.
    #[inline(always)]
    pub fn peek(&self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            Some(self.buffer[self.read_idx])
        }
    }

    /// Returns the number of readable samples currently buffered.
    #[inline(always)]
    pub fn available_read(&self) -> usize {
        self.count
    }

    /// Returns the remaining write capacity before the buffer is full.
    #[inline(always)]
    pub fn available_write(&self) -> usize {
        self.capacity.saturating_sub(self.count)
    }

    /// Returns the total cumulative count of underrun occurrences.
    #[inline(always)]
    pub fn underruns(&self) -> u64 {
        self.underruns
    }

    /// Returns the total cumulative count of underrun occurrences (alias for metrics).
    #[inline(always)]
    pub fn ring_buffer_underruns(&self) -> u64 {
        self.underruns
    }

    /// Total capacity of the circular buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Checks if the buffer contains zero readable samples.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Checks if the buffer is filled to capacity.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.count >= self.capacity
    }

    /// Clears all buffered samples and resets read and write pointers.
    pub fn clear(&mut self) {
        self.read_idx = 0;
        self.write_idx = 0;
        self.count = 0;
    }

    /// Resets the underrun counter to zero.
    pub fn reset_underruns(&mut self) {
        self.underruns = 0;
    }

    /// Pushes a slice of samples into the buffer.
    /// Returns the number of samples actually written.
    pub fn push_slice(&mut self, samples: &[T]) -> usize {
        let mut written = 0;
        for &s in samples {
            if self.push(s) {
                written += 1;
            } else {
                break;
            }
        }
        written
    }

    /// Pulls samples from the buffer into an output slice.
    /// If underruns occur, remaining slots are filled with silence (T::default()).
    /// Returns the number of valid samples pulled before underrun.
    pub fn pull_slice(&mut self, output: &mut [T]) -> usize {
        let mut valid = 0;
        for out in output.iter_mut() {
            if self.count > 0 {
                valid += 1;
            }
            *out = self.pull();
        }
        valid
    }
}
