// SPDX-License-Identifier: MPL-2.0
//! Circular buffer implementation for diagnostic event storage.
//!
//! This module provides a memory-bounded ring buffer that automatically
//! evicts the oldest entries when capacity is reached.

use std::collections::VecDeque;

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::diagnostics::buffer_capacity_bounds;
pub use crate::domain::diagnostics::BufferCapacity;

/// A generic circular buffer with fixed capacity.
///
/// When the buffer is full, pushing a new element evicts the oldest one.
/// Elements are stored in chronological order (oldest first).
///
/// # Example
///
/// ```
/// use iced_lens::diagnostics::{BufferCapacity, CircularBuffer};
///
/// // Use default capacity (1000 events)
/// let capacity = BufferCapacity::default();
/// let mut buffer: CircularBuffer<i32> = CircularBuffer::new(capacity);
///
/// buffer.push(1);
/// buffer.push(2);
/// buffer.push(3);
///
/// let items: Vec<_> = buffer.iter().copied().collect();
/// assert_eq!(items, vec![1, 2, 3]);
/// assert_eq!(buffer.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    /// Creates a new circular buffer with the specified capacity.
    #[must_use]
    pub fn new(capacity: BufferCapacity) -> Self {
        Self::with_raw_capacity(capacity.value())
    }

    /// Creates a new circular buffer with a raw capacity value.
    ///
    /// This is useful for testing with small capacities.
    /// For production use, prefer [`CircularBuffer::new`] with [`BufferCapacity`].
    #[must_use]
    pub fn with_raw_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1); // Ensure at least 1
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Pushes an element to the buffer, evicting the oldest if at capacity.
    pub fn push(&mut self, item: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    /// Returns an iterator over the elements in chronological order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    /// Returns the number of elements in the buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the maximum capacity of the buffer.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clears all elements from the buffer.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DEFAULT_DIAGNOSTICS_BUFFER_CAPACITY, MAX_DIAGNOSTICS_BUFFER_CAPACITY,
        MIN_DIAGNOSTICS_BUFFER_CAPACITY,
    };

    // BufferCapacity tests

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_eq!(buffer_capacity_bounds::MIN, MIN_DIAGNOSTICS_BUFFER_CAPACITY);
        assert_eq!(buffer_capacity_bounds::MAX, MAX_DIAGNOSTICS_BUFFER_CAPACITY);
        assert_eq!(
            buffer_capacity_bounds::DEFAULT,
            DEFAULT_DIAGNOSTICS_BUFFER_CAPACITY
        );
    }

    #[test]
    fn buffer_capacity_clamps_to_valid_range() {
        assert_eq!(
            BufferCapacity::new(0).value(),
            buffer_capacity_bounds::MIN
        );
        assert_eq!(
            BufferCapacity::new(100_000).value(),
            buffer_capacity_bounds::MAX
        );
    }

    #[test]
    fn buffer_capacity_accepts_valid_values() {
        assert_eq!(BufferCapacity::new(100).value(), 100);
        assert_eq!(BufferCapacity::new(1000).value(), 1000);
        assert_eq!(BufferCapacity::new(5000).value(), 5000);
    }

    #[test]
    fn buffer_capacity_default_returns_expected_value() {
        assert_eq!(
            BufferCapacity::default().value(),
            buffer_capacity_bounds::DEFAULT
        );
    }

    #[test]
    fn buffer_capacity_is_min_detects_minimum() {
        assert!(BufferCapacity::new(buffer_capacity_bounds::MIN).is_min());
        assert!(!BufferCapacity::new(1000).is_min());
    }

    #[test]
    fn buffer_capacity_is_max_detects_maximum() {
        assert!(BufferCapacity::new(buffer_capacity_bounds::MAX).is_max());
        assert!(!BufferCapacity::new(1000).is_max());
    }

    #[test]
    fn buffer_capacity_equality_works() {
        assert_eq!(BufferCapacity::new(500), BufferCapacity::new(500));
        assert_ne!(BufferCapacity::new(500), BufferCapacity::new(1000));
    }

    // CircularBuffer tests

    #[test]
    fn circular_buffer_push_and_retrieve() {
        let mut buffer: CircularBuffer<i32> = CircularBuffer::with_raw_capacity(5);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn circular_buffer_overflow_evicts_oldest() {
        let mut buffer: CircularBuffer<i32> = CircularBuffer::with_raw_capacity(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4); // Evicts 1
        buffer.push(5); // Evicts 2

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, vec![3, 4, 5]);
    }

    #[test]
    fn circular_buffer_iterator_chronological_order() {
        let mut buffer: CircularBuffer<i32> = CircularBuffer::with_raw_capacity(5);

        buffer.push(10);
        buffer.push(20);
        buffer.push(30);
        buffer.push(40);
        buffer.push(50);
        buffer.push(60); // Evicts 10

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, vec![20, 30, 40, 50, 60]);

        // Verify order is oldest to newest
        let mut prev = 0;
        for &item in buffer.iter() {
            assert!(item > prev);
            prev = item;
        }
    }

    #[test]
    fn circular_buffer_len_and_capacity() {
        let mut buffer: CircularBuffer<i32> = CircularBuffer::with_raw_capacity(5);

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 5);
        assert!(buffer.is_empty());

        buffer.push(1);
        buffer.push(2);

        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_empty());

        // Fill to capacity
        buffer.push(3);
        buffer.push(4);
        buffer.push(5);

        assert_eq!(buffer.len(), 5);

        // Overflow doesn't increase len
        buffer.push(6);
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn circular_buffer_clear() {
        let mut buffer: CircularBuffer<i32> = CircularBuffer::with_raw_capacity(5);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.len(), 3);

        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 5); // Capacity unchanged
    }

    #[test]
    fn circular_buffer_works_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Event {
            id: u32,
            name: String,
        }

        let mut buffer: CircularBuffer<Event> = CircularBuffer::with_raw_capacity(2);

        buffer.push(Event {
            id: 1,
            name: "first".to_string(),
        });
        buffer.push(Event {
            id: 2,
            name: "second".to_string(),
        });
        buffer.push(Event {
            id: 3,
            name: "third".to_string(),
        }); // Evicts first

        let events: Vec<_> = buffer.iter().collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, 2);
        assert_eq!(events[1].id, 3);
    }

    #[test]
    fn circular_buffer_new_uses_buffer_capacity() {
        let capacity = BufferCapacity::new(500);
        let buffer: CircularBuffer<i32> = CircularBuffer::new(capacity);
        assert_eq!(buffer.capacity(), 500);
    }
}
