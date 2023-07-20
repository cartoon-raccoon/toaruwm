//! A ringbuffer type used throughout toaruwm.
//!
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

use std::cmp::Ordering;
use std::collections::{vec_deque::IntoIter, VecDeque};
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
/// 
/// # Guarantees
/// 
/// Since so many structures within ToaruWM make use of a Ring,
/// it has to make certain guarantees about its behaviour that
/// these structures can rely on to make certain assumptions.
/// 
/// These guarantees are:
/// 
/// 1. The only time there is no focused item is when the ring is empty.
/// The focus is automatically set when the first item is pushed, and
/// unset when the last item is removed.
/// 
/// 2. The item pointed to by the focused item will not change unless
/// explicitly changed by the user, or the focused item is removed.
/// 
/// 3. The focus will always point to a valid item. It will never point
/// to an index that is out of bounds.
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
    /// Checks if the focus would wrap to the other end of
    /// the Ring if moved in the given direction.
    /// 
    /// Always returns false if no focus is set.
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

    /// Check if the given index points to a valid item.
    fn is_in_bounds(&self, idx: usize) -> bool {
        idx <= self.len() - 1
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
    /// 
    /// If the requested focus is out of bounds,
    /// it is set to the first element.
    /// 
    /// Is a no-op if the Ring is empty.
    #[inline(always)]
    pub fn set_focused(&mut self, idx: usize) {
        if !self.is_empty() {
            if self.is_in_bounds(idx) {
                self.focused = Some(idx);
            } else {
                self.focused = Some(0);
            }
        }
    }

    /// Unsets the focused element.
    #[inline(always)]
    fn unset_focused(&mut self) {
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

    /// Pushes an element to the front of the Ring.
    pub fn push(&mut self, item: T) {
        self.items.push_front(item);
        if let Some(idx) = self.focused {
            self.set_focused(idx + 1);
        } else {
            /* assume the ring was empty, and
            this is the first item */
            self.set_focused(0);
        }
    }

    /// Pushes an element to the back of the Ring.
    pub fn append(&mut self, item: T) {
        self.items.push_back(item);
        if self.focused.is_none() {
            /* assume the ring was empty, and
            this is the first item */
            self.set_focused(0)
        }
    }

    /// Pops the element off the front.
    /// 
    /// If the focus was on the front element,
    /// the focus moves to the next in line.
    /// 
    /// Returns None if the Ring is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        let ret = self.items.pop_front();
        if self.is_empty() {
            self.unset_focused();
        } else if let Some(focused) = self.focused {
            if focused > 0 {
                self.set_focused(focused - 1);
            }
        } else {
            unreachable!("focused not set with non-empty Ring")
        }
        ret
    }

    /// Moves the element at index `idx` to the front (index 0).
    /// 
    /// Is a no-op if `idx` is out of bounds.
    pub fn move_front(&mut self, idx: usize) {
        self.move_to(idx, 0);
    }

    /// Moves the element at index `from` to index `to`.
    /// 
    /// Is a no-op if `from` is out of bounds.
    pub fn move_to(&mut self, from: usize, to: usize) {
        if let Some(item) = self.remove(from) {
            self.insert(InsertPoint::Index(to), item);
        }
    }

    /// Removes an element from the Ring at `idx`, returning it if
    /// it exists.
    ///
    /// If the element removed is the focused element, the focus
    /// slides to the next element in line.
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        // if we can't access the element, immediately return
        let Some(ret) = self.items.remove(idx) else {
            return None
        };

        // if empty, unset our focus
        if self.items.is_empty() {
            self.unset_focused();
            return Some(ret)
        }

        /* do focus checks */
        if let Some(f_idx) = self.focused {
            if idx < f_idx {
                /* we removed an element in front of the focused
                element.
                in order to maintain our guarantee, we need to 
                change the focus to continue to point to the
                same element.
                slide the focus down by one.

                this code won't panic, since idx
                is strictly less than f_idx, and
                idx >= 0, so f_idx >= 1. */
                self.set_focused(f_idx - 1)
            }

            /* we don't need to account for cases where
            idx > f_idx, as the focused item won't change
            in that case. 
            if idx == f_idx, then we've removed the
            focused item, and the focus should slide
            to the next in line.*/

            if !self.is_in_bounds(f_idx) {
                /* if our focus now points out of bounds,
                that means we were already pointing to the
                end of the deque before removal.
                wrap around to the front. */
                self.set_focused(0);
            }
        } else if !self.is_empty() {
            /* if we've reached this point, then
            we've definitely had something to remove
            but somehow we have None as our focus.
            this means our guarantees are not upheld. */
            unreachable!("focus not set with non-empty ring")
        }
        Some(ret)
    }

    /// Insert an item into the Ring with an insert point.
    /// 
    /// If the Ring is empty, it pushes the item, disregarding the
    /// insert point.
    ///
    /// If insert point revolves around the focused item and nothing has focus,
    /// it appends the item to the end of the ring.
    /// 
    /// If the insert point is the focused item, then the inserted
    /// item becomes the focus, replacing whatever was in focus, which
    /// gets slid up by 1.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use toaruwm::core::{Ring, ring::InsertPoint};
    /// 
    /// let mut ring = Ring::new();
    /// for i in 1..10 {
    ///     ring.append(i);
    /// }
    /// // [1, 2, 3, 4, 5, 6, 7, 8, 9]
    /// //  ^
    /// 
    /// ring.insert(InsertPoint::BeforeFocused, 10);
    /// //[10, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    /// //     ^
    /// assert_eq!(*ring.focused().expect("no focus"), 1);
    /// 
    /// ring.insert(InsertPoint::Focused, 69);
    /// // [10, 69, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    /// //      ^
    /// assert_eq!(*ring.focused().expect("no focus"), 69);
    /// ```
    pub fn insert(&mut self, point: InsertPoint, item: T) {
        use Direction::*;
        use InsertPoint::*;

        if self.is_empty() {
            self.push(item);
            return
        }

        debug_assert!(self.focused.is_some());
        let f_idx = self.focused.unwrap();

        match point {
            Index(idx) => {
                if !self.is_in_bounds(idx) {
                    /* fail silently */
                    return
                }
                self.items.insert(idx, item);
                if idx <= f_idx {
                    /* we inserted at or before
                    the focused item, so to
                    maintain our guarantee, we
                    have to slide the index up by 1 */
                    debug_assert!(self.is_in_bounds(f_idx + 1));
                    self.set_focused(f_idx + 1);
                }
            }
            Focused => {
                self.items.insert(f_idx, item);
                /* don't change the focus idx, since this
                now replaces the previous focus */
            }
            AfterFocused => { // we must preserve the focus here
                debug_assert!(self.is_in_bounds(f_idx));
                if self.would_wrap(Forward) {
                    self.append(item);
                } else {
                    self.insert(InsertPoint::Index(f_idx + 1), item);
                }
            }
            BeforeFocused => { // we must preserve the focus here
                debug_assert!(self.is_in_bounds(f_idx));
                if self.would_wrap(Backward) {
                    self.push(item);
                } else {
                    self.insert(InsertPoint::Index(f_idx - 1), item);
                }
            }
            First => {
                self.push(item);
            }
            Last => {
                self.append(item);
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

    /// Sorts a Ring internally.
    ///
    /// ## Note
    ///
    /// This internally rearranges the storage of the Ring
    /// into a contiguous block of memory, which can be
    /// very computationally intensive!
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        let cont = self.items.make_contiguous();
        cont.sort_by(f);
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
    /// Is a no-op if nothing is in focus or the ring only has one item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::core::Ring;
    /// use toaruwm::types::Direction::*;
    ///
    /// let mut ring1 = Ring::with_capacity(2);
    ///
    /// ring1.append(1); ring1.append(2);
    /// ring1.set_focused(0);
    ///
    /// ring1.cycle_focus(Forward);
    ///
    /// # //assert_eq!(ring1.focused_idx().unwrap(), 1);
    ///
    /// assert_eq!(*ring1.focused().unwrap(), 2);
    ///
    /// let mut ring2 = Ring::with_capacity(1);
    ///
    /// ring2.append(1);
    /// ring2.set_focused(0);
    ///
    /// ring2.cycle_focus(Forward);
    ///
    /// # //assert_eq!(ring2.focused_idx().unwrap(), 0);
    ///
    /// assert_eq!(*ring2.focused().unwrap(), 1);
    /// ```
    pub fn cycle_focus(&mut self, direction: Direction) {
        use Direction::*;

        if self.len() <= 1 {
            return;
        }

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

impl<T: Ord> Ring<T> {
    /// Reorganizes the inner Ring storage
    /// and then sorts the Ring using `slice::sort`.
    pub fn sort(&mut self) {
        let cont = self.items.make_contiguous();
        cont.sort();
    }

    /// Reorganizes the inner Ring storage
    /// and then sorts the Ring using `slice::sort_unstable`.
    pub fn sort_unstable(&mut self) {
        let cont = self.items.make_contiguous();
        cont.sort_unstable();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ring_focus() {
        let mut ring = Ring::new();

        assert!(ring.focused().is_none());

        ring.push(1); 
        // [1]
        //  ^
        assert_eq!(*ring.focused().expect("no focus"), 1);

        ring.push(2);
        // [2, 1]
        //     ^
        assert_eq!(*ring.focused().expect("no focus"), 1);

        ring.push(3);
        // [3, 2, 1]
        //        ^
        assert_eq!(*ring.focused().expect("no focus"), 1);

        ring.cycle_focus(Direction::Forward);
        // [3, 2, 1]
        //  ^
        assert_eq!(*ring.focused().expect("no_focus"), 3);

        ring.append(4);
        // [3, 2, 1, 4]
        //  ^
        assert_eq!(*ring.focused().expect("no focus"), 3);

        ring.cycle_focus(Direction::Backward);
        // [3, 2, 1, 4]
        //           ^
        assert_eq!(*ring.focused().expect("no focus"), 4);
    }

    #[test]
    fn test_ring_insert() {
        let mut ring = Ring::new();
        for i in 1..10 {
            ring.append(i);
        }
        // [1, 2, 3, 4, 5, 6, 7, 8, 9]
        //  ^

        ring.insert(InsertPoint::BeforeFocused, 10);
        // [10, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        //      ^
        assert_eq!(ring[0], 10);
        assert_eq!(*ring.focused().expect("no focus"), 1);

        ring.insert(InsertPoint::Focused, 69);
        // [10, 69, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        //      ^
        assert_eq!(ring[1], 69);
        assert_eq!(*ring.focused().expect("no focus"), 69);

        ring.insert(InsertPoint::AfterFocused, 15);
        // [10, 69, 15, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        //      ^
        assert_eq!(ring[2], 15);
        assert_eq!(*ring.focused().expect("no focus"), 69);

        // insertion at index but the index happens to be the focus
        ring.insert(InsertPoint::Index(1), 20);
        // [10, 20, 69, 15, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        //          ^
        assert_eq!(ring[1], 20);
        assert_eq!(*ring.focused().expect("no focus"), 69);
    }

    #[test]
    fn test_ring_removal() {
        let mut ring = Ring::new();

        for i in 1..10 {
            ring.append(i);
        }
        // [1, 2, 3, 4, 5, 6, 7, 8, 9]
        //  ^

        ring.remove(0);
        // [2, 3, 4, 5, 6, 7, 8, 9]
        //  ^
        assert_eq!(ring[0], 2);
        assert_eq!(*ring.focused().expect("no focus"), 2);

        ring.set_focused(3);
        // [2, 3, 4, 5, 6, 7, 8, 9]
        //           ^
        assert_eq!(*ring.focused().expect("no focus"), 5);

        ring.remove(1);
        // [2, 4, 5, 6, 7, 8, 9]
        //        ^
        assert_eq!(ring[1], 4);
        assert_eq!(*ring.focused().expect("no focus"), 5);

        for _ in 0..7 {
            ring.remove(0);
        }

        assert!(ring.focused().is_none());
    }

    #[test]
    fn test_ring_move() {
        let mut ring = Ring::new();

        for i in 1..10 {
            ring.append(i);
        }
        // [1, 2, 3, 4, 5, 6, 7, 8, 9]
        //  ^

        ring.move_to(7, 3);
        // [1, 2, 3, 8, 4, 5, 6, 7, 9]
        //  ^
        assert_eq!(*ring.focused().expect("no focus"), 1);
        assert_eq!(ring[3], 8);
    }
}
