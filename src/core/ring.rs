use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::types::Direction;

pub enum InsertPoint {
    Index(usize),
    Focused,
    AfterFocused,
    BeforeFocused,
    First,
    Last,
}

pub enum Selector<'a, T> {
    Any,
    Focused,
    Index(usize),
    Condition(&'a dyn Fn(&T) -> bool),
}

pub struct Ring<T> {
    items: VecDeque<T>,
    focused: Option<usize>,
}

impl <T> Ring<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            focused: None,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(cap),
            focused: None,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

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
                        return true
                    }
                }
            }
            Backward => {
                if let Some(i) = self.focused {
                    if i == 0 {
                        return true
                    }
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn focused_idx(&self) -> Option<usize> {
        self.focused
    }

    pub fn move_front(&mut self, idx: usize) {
        if idx != 0 {self.items.swap(0, idx)}
    }

    pub fn push(&mut self, item: T) {
        self.items.push_front(item)
    }

    pub fn append(&mut self, item: T) {
        self.items.push_back(item)
    }

    pub fn pop(&mut self, idx: usize) -> T {
        self.move_front(idx);

        self.items.pop_front().unwrap()
    }

    /// Insert an item into the Ring with an insert point
    /// 
    /// If insert point revolves around the focused item and nothing has focus,
    /// it appends the item to the end of the ring.
    pub fn insert(&mut self, point: InsertPoint, item: T) {
        use InsertPoint::*;
        use Direction::*;

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

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.items.remove(idx)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.items.get_mut(idx)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &T> {
        self.items.iter().rev()
    }

    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().rev()
    }

    /// Rotates the entire buffer by 1 in the given direction.
    /// 
    /// Preserves the the focus.
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

    #[inline(always)]
    pub fn unset_focused(&mut self) {
        self.focused = None
    }

    pub fn focused(&self) -> Option<&T> {
        if let Some(i) = self.focused {
            return self.get(i)
        }

        None
    }

    pub fn focused_mut(&mut self) -> Option<&mut T> {
        if let Some(i) = self.focused {
            return self.get_mut(i)
        }

        None
    }

    pub fn set_focused(&mut self, idx: usize) {
        self.focused = Some(idx);
    }

    pub fn apply_to<F: FnMut(&mut T)>(&mut self, s: Selector<'_, T>, mut f: F) {
        if let Some(idx) = self.index(s) {
            f(&mut self[idx])
        }
    }

    fn element_by(&self, cond: impl Fn(&T) -> bool) -> Option<(usize, &T)> {
        self.iter().enumerate().find(|(_, e)| cond(*e))
    }

    #[allow(dead_code)]
    fn element_by_mut(&mut self, cond: impl Fn(&T) -> bool) -> Option<(usize, &mut T)> {
        self.iter_mut().enumerate().find(|(_, e)| cond(*e))
    }

    pub fn index(&self, s: Selector<'_, T>) -> Option<usize> {
        use Selector::*;

        match s {
            Any | Focused => {
                self.focused
            }
            Index(i) => {
                if i < self.len() {
                    Some(i)
                } else {
                    None
                }
            }
            Condition(f) => {
                self.element_by(f).map(|(i, _)| i)
            }
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