//! Discrete multi-valued logic types, identifiers, and event queue primitives
//! adhering to IEEE 1164 for mixed-signal co-simulation.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt;

/// IEEE 1164 standard 9-value logic level.
///
/// Values:
/// - `U`: Uninitialized
/// - `X`: Forcing Unknown
/// - `0`: Forcing Low (0)
/// - `1`: Forcing High (1)
/// - `Z`: High Impedance
/// - `W`: Weak Unknown
/// - `L`: Weak Low (0)
/// - `H`: Weak High (1)
/// - `-`: Don't care
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum LogicLevel {
    #[default]
    U = 0,
    X = 1,
    Zero = 2,
    One = 3,
    Z = 4,
    W = 5,
    L = 6,
    H = 7,
    DontCare = 8,
}

impl LogicLevel {
    /// Returns the standard character representation of this logic level.
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            Self::U => 'U',
            Self::X => 'X',
            Self::Zero => '0',
            Self::One => '1',
            Self::Z => 'Z',
            Self::W => 'W',
            Self::L => 'L',
            Self::H => 'H',
            Self::DontCare => '-',
        }
    }

    /// Parses a single character into a LogicLevel.
    #[inline]
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'U' => Some(Self::U),
            'X' => Some(Self::X),
            '0' => Some(Self::Zero),
            '1' => Some(Self::One),
            'Z' => Some(Self::Z),
            'W' => Some(Self::W),
            'L' => Some(Self::L),
            'H' => Some(Self::H),
            '-' => Some(Self::DontCare),
            _ => None,
        }
    }

    /// Converts the logic state to a standard boolean if definitively 0 or 1 (strong or weak).
    #[inline]
    pub fn to_bool(self) -> Option<bool> {
        match self {
            Self::Zero | Self::L => Some(false),
            Self::One | Self::H => Some(true),
            _ => None,
        }
    }

    /// Returns `true` if the state represents high (strong or weak 1).
    #[inline]
    pub fn is_high(self) -> bool {
        matches!(self, Self::One | Self::H)
    }

    /// Returns `true` if the state represents low (strong or weak 0).
    #[inline]
    pub fn is_low(self) -> bool {
        matches!(self, Self::Zero | Self::L)
    }

    /// Returns `true` if the signal is in the high-impedance state (`Z`).
    #[inline]
    pub fn is_high_z(self) -> bool {
        matches!(self, Self::Z)
    }

    /// Returns `true` if the signal has a strong drive strength (`0` or `1`).
    #[inline]
    pub fn is_strong(self) -> bool {
        matches!(self, Self::Zero | Self::One)
    }

    /// Returns `true` if the signal has a weak drive strength (`L` or `H` or `W`).
    #[inline]
    pub fn is_weak(self) -> bool {
        matches!(self, Self::L | Self::H | Self::W)
    }

    /// IEEE 1164 NOT operation.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        !self
    }

    /// IEEE 1164 AND operation table.
    pub fn and(self, other: Self) -> Self {
        use LogicLevel::*;
        // IEEE 1164 AND table
        // Columns/Rows: U, X, 0, 1, Z, W, L, H, -
        const AND_TABLE: [[LogicLevel; 9]; 9] = [
            // U
            [U, U, Zero, U, U, U, Zero, U, U],
            // X
            [U, X, Zero, X, X, X, Zero, X, X],
            // 0
            [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero],
            // 1
            [U, X, Zero, One, X, X, Zero, One, X],
            // Z
            [U, X, Zero, X, X, X, Zero, X, X],
            // W
            [U, X, Zero, X, X, X, Zero, X, X],
            // L
            [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero],
            // H
            [U, X, Zero, One, X, X, Zero, One, X],
            // -
            [U, X, Zero, X, X, X, Zero, X, X],
        ];
        AND_TABLE[self as usize][other as usize]
    }

    /// IEEE 1164 OR operation table.
    pub fn or(self, other: Self) -> Self {
        use LogicLevel::*;
        const OR_TABLE: [[LogicLevel; 9]; 9] = [
            // U
            [U, U, U, One, U, U, U, One, U],
            // X
            [U, X, X, One, X, X, X, One, X],
            // 0
            [U, X, Zero, One, X, X, Zero, One, X],
            // 1
            [One, One, One, One, One, One, One, One, One],
            // Z
            [U, X, X, One, X, X, X, One, X],
            // W
            [U, X, X, One, X, X, X, One, X],
            // L
            [U, X, Zero, One, X, X, Zero, One, X],
            // H
            [One, One, One, One, One, One, One, One, One],
            // -
            [U, X, X, One, X, X, X, One, X],
        ];
        OR_TABLE[self as usize][other as usize]
    }

    /// IEEE 1164 XOR operation table.
    pub fn xor(self, other: Self) -> Self {
        use LogicLevel::*;
        const XOR_TABLE: [[LogicLevel; 9]; 9] = [
            // U
            [U, U, U, U, U, U, U, U, U],
            // X
            [U, X, X, X, X, X, X, X, X],
            // 0
            [U, X, Zero, One, X, X, Zero, One, X],
            // 1
            [U, X, One, Zero, X, X, One, Zero, X],
            // Z
            [U, X, X, X, X, X, X, X, X],
            // W
            [U, X, X, X, X, X, X, X, X],
            // L
            [U, X, Zero, One, X, X, Zero, One, X],
            // H
            [U, X, One, Zero, X, X, One, Zero, X],
            // -
            [U, X, X, X, X, X, X, X, X],
        ];
        XOR_TABLE[self as usize][other as usize]
    }

    /// IEEE 1164 NAND operation.
    #[inline]
    pub fn nand(self, other: Self) -> Self {
        self.and(other).not()
    }

    /// IEEE 1164 NOR operation.
    #[inline]
    pub fn nor(self, other: Self) -> Self {
        self.or(other).not()
    }

    /// IEEE 1164 XNOR operation.
    #[inline]
    pub fn xnor(self, other: Self) -> Self {
        self.xor(other).not()
    }

    /// IEEE 1164 Resolution function for multiple drivers on a single wire / bus.
    pub fn resolve(self, other: Self) -> Self {
        use LogicLevel::*;
        const RESOLUTION_TABLE: [[LogicLevel; 9]; 9] = [
            // U
            [U, U, U, U, U, U, U, U, U],
            // X
            [U, X, X, X, X, X, X, X, X],
            // 0
            [U, X, Zero, X, Zero, Zero, Zero, Zero, X],
            // 1
            [U, X, X, One, One, One, One, One, X],
            // Z
            [U, X, Zero, One, Z, W, L, H, X],
            // W
            [U, X, Zero, One, W, W, W, W, X],
            // L
            [U, X, Zero, One, L, W, L, W, X],
            // H
            [U, X, Zero, One, H, W, W, H, X],
            // -
            [U, X, X, X, X, X, X, X, X],
        ];
        RESOLUTION_TABLE[self as usize][other as usize]
    }
}

impl fmt::Display for LogicLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl From<bool> for LogicLevel {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Self::One
        } else {
            Self::Zero
        }
    }
}

impl std::ops::Not for LogicLevel {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Self::U => Self::U,
            Self::X => Self::X,
            Self::Zero => Self::One,
            Self::One => Self::Zero,
            Self::Z => Self::X,
            Self::W => Self::X,
            Self::L => Self::One,
            Self::H => Self::Zero,
            Self::DontCare => Self::X,
        }
    }
}

impl std::ops::BitAnd for LogicLevel {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl std::ops::BitOr for LogicLevel {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

impl std::ops::BitXor for LogicLevel {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}

/// Unique identifier for a digital signal net in mixed-signal co-simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DigitalNodeId(pub u32);

impl DigitalNodeId {
    /// Creates a new `DigitalNodeId`.
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the integer index of the digital node.
    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for DigitalNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DNode({})", self.0)
    }
}

/// A scheduled discrete digital state transition event.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DigitalEvent {
    /// Scheduled continuous simulation time (in seconds).
    pub time: f64,
    /// Target digital signal node.
    pub node: DigitalNodeId,
    /// New logic level to apply.
    pub level: LogicLevel,
    /// Monotonic sequence priority for deterministic tie-breaking of simultaneous events.
    pub priority: u64,
}

impl Eq for DigitalEvent {}

impl Ord for DigitalEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap ordering: earlier time is Greater so BinaryHeap pops earliest first.
        other
            .time
            .total_cmp(&self.time)
            .then_with(|| other.priority.cmp(&self.priority))
    }
}

impl PartialOrd for DigitalEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority event queue for discrete event-driven logic simulation.
#[derive(Debug, Clone, Default)]
pub struct EventQueue {
    heap: BinaryHeap<DigitalEvent>,
    next_priority: u64,
}

impl EventQueue {
    /// Creates an empty event queue.
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            next_priority: 0,
        }
    }

    /// Schedules a state transition at the specified timestamp.
    pub fn schedule(&mut self, time: f64, node: DigitalNodeId, level: LogicLevel) -> u64 {
        let priority = self.next_priority;
        self.next_priority = self.next_priority.wrapping_add(1);
        self.heap.push(DigitalEvent {
            time,
            node,
            level,
            priority,
        });
        priority
    }

    /// Pushes a pre-constructed event into the queue.
    pub fn push(&mut self, event: DigitalEvent) {
        if event.priority >= self.next_priority {
            self.next_priority = event.priority.wrapping_add(1);
        }
        self.heap.push(event);
    }

    /// Pops the earliest scheduled event from the queue.
    pub fn pop(&mut self) -> Option<DigitalEvent> {
        self.heap.pop()
    }

    /// Peeks at the earliest scheduled event without removing it.
    pub fn peek(&self) -> Option<&DigitalEvent> {
        self.heap.peek()
    }

    /// Returns the timestamp of the next scheduled event, if any.
    pub fn next_event_time(&self) -> Option<f64> {
        self.heap.peek().map(|e| e.time)
    }

    /// Number of scheduled events remaining in the queue.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns true if there are no pending events.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Clears all pending events from the queue.
    pub fn clear(&mut self) {
        self.heap.clear();
        self.next_priority = 0;
    }

    /// Drains and returns all events scheduled at or before `time + epsilon`.
    pub fn drain_events_at(&mut self, time: f64, epsilon: f64) -> Vec<DigitalEvent> {
        let mut events = Vec::new();
        let cutoff = time + epsilon;
        while let Some(e) = self.peek() {
            if e.time <= cutoff {
                if let Some(popped) = self.pop() {
                    events.push(popped);
                }
            } else {
                break;
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_level_operations() {
        assert_eq!(!LogicLevel::Zero, LogicLevel::One);
        assert_eq!(!LogicLevel::One, LogicLevel::Zero);
        assert_eq!(LogicLevel::Zero & LogicLevel::One, LogicLevel::Zero);
        assert_eq!(LogicLevel::One & LogicLevel::One, LogicLevel::One);
        assert_eq!(LogicLevel::Zero | LogicLevel::One, LogicLevel::One);
        assert_eq!(LogicLevel::One ^ LogicLevel::One, LogicLevel::Zero);
        assert_eq!(LogicLevel::One ^ LogicLevel::Zero, LogicLevel::One);
        assert_eq!(LogicLevel::L.to_bool(), Some(false));
        assert_eq!(LogicLevel::H.to_bool(), Some(true));
        assert_eq!(LogicLevel::Z.to_bool(), None);
    }

    #[test]
    fn test_logic_resolution() {
        // High-Z resolution
        assert_eq!(LogicLevel::Z.resolve(LogicLevel::One), LogicLevel::One);
        assert_eq!(LogicLevel::Zero.resolve(LogicLevel::Z), LogicLevel::Zero);
        // Bus contention
        assert_eq!(LogicLevel::Zero.resolve(LogicLevel::One), LogicLevel::X);
        // Weak vs strong
        assert_eq!(LogicLevel::Zero.resolve(LogicLevel::H), LogicLevel::Zero);
        assert_eq!(LogicLevel::L.resolve(LogicLevel::One), LogicLevel::One);
    }

    #[test]
    fn test_event_queue_priority() {
        let mut q = EventQueue::new();
        q.schedule(1.5, DigitalNodeId(1), LogicLevel::One);
        q.schedule(0.5, DigitalNodeId(2), LogicLevel::Zero);
        q.schedule(1.0, DigitalNodeId(3), LogicLevel::H);

        assert_eq!(q.next_event_time(), Some(0.5));
        let e1 = q.pop().unwrap();
        assert_eq!(e1.node, DigitalNodeId(2));
        assert_eq!(e1.time, 0.5);

        let e2 = q.pop().unwrap();
        assert_eq!(e2.node, DigitalNodeId(3));
        assert_eq!(e2.time, 1.0);

        let e3 = q.pop().unwrap();
        assert_eq!(e3.node, DigitalNodeId(1));
        assert_eq!(e3.time, 1.5);

        assert!(q.is_empty());
    }
}
