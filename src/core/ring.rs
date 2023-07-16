//! This module defines Ring, a data structure that presents a
//! ringbuffer-like interface.
//!
//! A `Ring` abstracts over an internal buffer and presents an interface
//! that resembles a ring-buffer, with one element in focus, or none.
//! It can be rotated and the focus can be set, unset or cycled through
//! in different directions.
//!
//! Retrieving items from a `Ring` can be done using a `Selector`, which
//! can retrieve the focused item, an item at an index, or an item that
//! fulfills a predicate.
//!
//! Insertion into a Ring is done with an InsertPoint, which can insert an item
//! with respect to the current item in focus, or at a specified index.

use core::ops::{Index, IndexMut};

use std::collections::{VecDeque, vec_deque::IntoIter};
use std::iter::FromIterator;

use custom_debug_derive::Debug;

use super::types::Direction;

/// A point at which to insert an element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InsertPoint {
    /// At a given index.
    Index(usize),
    /// Insert at the focused element, taking its place.
    Focused,
    /// Insert after the focused element.
    AfterFocused,
    /// Insert before the focused element.
    BeforeFocused,
    /// Insert at the first index.
    First,
    /// Insert at the back.
    Last,
}

/// A type to select items from a Ring.
#[derive(Debug, Clone, Copy)]
pub enum Selector<'a, T> {
    /// Any item.
    Any,
    /// The focused item.
    Focused,
    /// At a specific index.
    Index(usize),
    /// Whichever item fulfills a given predicate.
    Condition(&'a dyn Fn(&T) -> bool),
}

/// An internal data structure to manage items as a ring buffer.
///
/// Provides an interface where the data is a ring of items,
/// with a single item in focus.
/// Ideally, the only time there is no focused item is when the ring is empty.
#[derive(Clone, Debug, Default)]
pub struct Ring<T> {
    pub(crate) items: VecDeque<T>,
    pub(crate) focused: Option<usize>,
}

impl<T> Ring<T> {
    /// Construct a new instance of a Ring.
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            focused: None,
        }
    }

    /// Construct a new instance of a Ring with a set capacity
    /// already allocated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use toaruwm::core::Ring;
    ///
    /// let ring: Ring<u32> = Ring::with_capacity(2);
    ///
    /// assert_eq!(2, ring.capacity());
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(cap),
            focused: None,
        }
    }

    /// Number of items already inside the Ring.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use toaruwm::core::Ring;
    ///
    /// let mut ring: Ring<u32> = Ring::new();
    ///
    /// ring.push(5);
    /// ring.push(9);
    /// ring.push(4);
    ///
    /// assert_eq!(ring.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the number of items the Ring can hold
    /// without reallocating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use toaruwm::core::Ring;
    ///
    /// let ring: Ring<u32> = Ring::with_capacity(2);
    ///
    /// assert_eq!(2, ring.capacity());
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Returns whether the Ring is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use toaruwm::core::Ring;
    ///
    /// let ring: Ring<u32> = Ring::new();
    ///
    /// assert!(ring.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    fn would_wrap(&self, direction: Direction) -> bool {
        use Direction::*;

        match direction {
            Forward => {
                if let Some(i) = self.focused {
                    if i == self.len() - 1 {
                        return true;
                    }
                }
            }
            Backward => {
                if let Some(i) = self.focused {
                    if i == 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns the index of the element that is
    /// currently in focus.
    ///
    ///
    #[inline(always)]
    pub fn focused_idx(&self) -> Option<usize> {
        self.focused
    }

    /// Sets the element to be in focus by index.
    #[inline(always)]
    pub fn set_focused(&mut self, idx: usize) {
        self.focused = Some(idx);
    }

    /// Unsets the focused element.
    #[inline(always)]
    pub fn unset_focused(&mut self) {
        self.focused = None
    }

    /// Returns a reference to the focused element.
    pub fn focused(&self) -> Option<&T> {
        if let Some(i) = self.focused {
            return self.get(i);
        }

        None
    }

    /// Returns a mutable reference to the focused element.
    pub fn focused_mut(&mut self) -> Option<&mut T> {
        if let Some(i) = self.focused {
            return self.get_mut(i);
        }

        None
    }

    /// Moves the element specified by index to the front.
    pub fn move_front(&mut self, idx: usize) {
        if idx != 0 {
            self.items.swap(0, idx)
        }
    }

    /// Pushes an element to the front of the Ring.
    pub fn push(&mut self, item: T) {
        self.items.push_front(item);
        if let Some(idx) = self.focused {
            self.set_focused(idx + 1)
        }
    }

    /// Pushes an element to the back of the Ring.
    pub fn append(&mut self, item: T) {
        self.items.push_back(item)
    }

    /// Pops the element off the front.
    ///
    /// # Panics
    ///
    /// Panics if the Ring is empty.
    pub fn pop_front(&mut self) -> T {
        if let Some(f) = self.focused {
            if f == 0 {
                self.unset_focused();
            }
        }
        self.items.pop_front().unwrap()
    }

    /// Removes an element from the Ring, returning it if it exists.
    ///
    /// If the item being removed is the focused element,
    /// it also unsets the focus.
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        let Some(ret) = self.items.remove(idx) else {
            return None
        };

        // check if we just removed the focused element.
        if let Some(i) = self.focused {
            if i == idx {
                self.unset_focused();
            }
        }
        // if empty, unfocus
        if self.is_empty() {
            self.unset_focused();
        }

        Some(ret)
    }

    /// Insert an item into the Ring with an insert point.
    ///
    /// The focused index does not change.
    ///
    /// If insert point revolves around the focused item and nothing has focus,
    /// it appends the item to the end of the ring.
    pub fn insert(&mut self, point: InsertPoint, item: T) {
        use Direction::*;
        use InsertPoint::*;

        match point {
            Index(idx) => {
                // don't bother checking for whether it would wrap or not
                self.items.insert(idx, item);
            }
            Focused => {
                if let Some(idx) = self.focused {
                    self.items.insert(idx, item);
                } else {
                    self.append(item);
                }
            }
            AfterFocused => {
                if let Some(idx) = self.focused {
                    if self.would_wrap(Forward) {
                        self.items.push_back(item);
                    } else {
                        self.items.insert(idx + 1, item);
                    }
                } else {
                    self.append(item);
                }
            }
            BeforeFocused => {
                if let Some(idx) = self.focused {
                    if self.would_wrap(Backward) {
                        self.items.push_back(item);
                    } else {
                        self.items.insert(idx - 1, item);
                    }
                } else {
                    self.append(item);
                }
            }
            First => {
                self.items.push_front(item);
            }
            Last => {
                self.items.push_back(item);
            }
        }
    }

    /// Returns a reference to the item at the specified index.
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    /// Returns a mutable reference to the item at the specified index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.items.get_mut(idx)
    }

    /// Returns an iterator over the items in the Ring.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Returns a mutable iterator over the items in the Ring.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    /// Returns a reversed iterator over the items in the Ring.
    ///
    /// Equivalent to calling `Ring::iter().rev()`.
    pub fn iter_rev(&self) -> impl Iterator<Item = &T> {
        self.items.iter().rev()
    }

    /// Returns a reversed mutable iterator over the items in the Ring.
    ///
    /// Equivalent to calling `Ring::iter_mut().rev()`.
    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().rev()
    }

    /// Rotates the entire buffer by 1 in the given direction.
    /// The focus is rotated with the buffer, so it points at the
    /// same element.
    pub fn rotate(&mut self, direction: Direction) {
        self.rotate_by(1, direction);
        self.cycle_focus(direction);
    }

    pub(crate) fn rotate_by(&mut self, step: usize, direction: Direction) {
        use Direction::*;
        match direction {
            Forward => {
                self.items.rotate_right(step);
            }
            Backward => {
                self.items.rotate_left(step);
            }
        }
    }

    /// Cycles the focus by one in the given direction.
    ///
    /// Is a no-op if nothing is in focus.
    pub fn cycle_focus(&mut self, direction: Direction) {
        use Direction::*;

        match direction {
            Forward => {
                if let Some(i) = self.focused {
                    if self.would_wrap(Forward) {
                        self.focused = Some(0)
                    } else {
                        self.focused = Some(i + 1)
                    }
                }
            }
            Backward => {
                if let Some(i) = self.focused {
                    if self.would_wrap(Backward) {
                        self.focused = Some(self.len() - 1)
                    } else {
                        self.focused = Some(i - 1)
                    }
                }
            }
        }
    }

    /// Applies a closure to the selected index.
    pub fn apply_to<F: FnMut(&mut T)>(&mut self, s: Selector<'_, T>, mut f: F) {
        if let Some(idx) = self.index(s) {
            f(&mut self[idx])
        }
    }

    /// Gets a reference to an element that satisfies a given closure.
    pub fn element_by(&self, cond: impl Fn(&T) -> bool) -> Option<(usize, &T)> {
        self.iter().enumerate().find(|(_, e)| cond(*e))
    }

    /// Gets a mutable reference to an element that satisfies a given closure.
    pub fn element_by_mut(&mut self, cond: impl Fn(&T) -> bool) -> Option<(usize, &mut T)> {
        self.iter_mut().enumerate().find(|(_, e)| cond(*e))
    }

    /// Get the index of an element via a selector.
    pub fn index(&self, s: Selector<'_, T>) -> Option<usize> {
        use Selector::*;

        match s {
            Any | Focused => self.focused,
            Index(i) => {
                if i < self.len() {
                    Some(i)
                } else {
                    None
                }
            }
            Condition(f) => self.element_by(f).map(|(i, _)| i),
        }
    }
}

impl<T> Index<usize> for Ring<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.items[idx]
    }
}

impl<T> IndexMut<usize> for Ring<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[idx]
    }
}

impl<T> FromIterator<T> for Ring<T> {
    fn from_iter<A>(iter: A) -> Self
    where
        A: IntoIterator<Item = T>,
    {
        Ring {
            items: iter.into_iter().collect(),
            focused: None,
        }
    }
}

impl<T> IntoIterator for Ring<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
