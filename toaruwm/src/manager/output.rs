use crate::types::{Rectangle, Point, Size, Physical, Cardinal};

/// A layout of outputs, as defined by the user.
#[derive(Debug, Default)]
pub struct OutputLayout {
    pub(crate) root: Option<OutputNode>
}

impl OutputLayout {
    /// Creates a new OutputLayout.
    pub fn new() -> Self {
        Self {root: None}
    }

    /// Checks if the OutputLayout is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of Outputs in the OutputLayout.
    pub fn size(&self) -> usize {
        let Some(tree) = &self.root else {
            return 0
        };

        tree.size()
    }

    /// Creates a new OutputLayout with the provided output.
    pub fn with_output(output: Output) -> Self {
        Self {
            root: Some(output.into_outputnode(Point::zeroed(), None))
        }
    }

    /// Insert an output at a specified Point.
    /// 
    /// `point` is ignored and `output` is inserted at (0,0) if `self` is currently empty.
    pub fn insert_at_point(&mut self, point: Point<Physical>, output: Output) -> Result<(), Output> {
        let Some(tree) = &mut self.root else {
            self.insert_first(output);
            return Ok(())
        };

        // add output as a direct child of our root
        tree.add_child_direct(point, None, output);
        Ok(())
    }

    /// Insert an output into the Layout with respect to another already-inserted output.
    /// 
    /// `name` is ignored, and `output` is inserted at (0,0) if `self` is currently empty.
    pub fn insert_relative_to<S: AsRef<str>>(&mut self, name: S, card: Cardinal, output: Output) -> Result<(), Output> {
        let Some(tree) = &mut self.root else {
            self.insert_first(output);
            return Ok(())
        };

        tree.add_child_of(name.as_ref(), card, output)
    }

    /// Insert `output` as a mirror of the output with `name`.
    /// 
    /// `name` is ignored and `output` is inserted at (0,0) with no mirroring, if `self` is currently empty.
    pub fn insert_mirror<S: AsRef<str>>(&mut self, name: S, output: Output) -> Result<(), Output> {
        let Some(tree) = &mut self.root else {
            self.insert_first(output);
            return Ok(())
        };

        todo!()
    }

    fn insert_first(&mut self, output: Output) {
        self.root = Some(output.into_outputnode(Point::zeroed(), None));
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct OutputNode {
    pub name: String,
    pub position: Rectangle<Physical>,
    pub refresh: i32,
    pub scale: f32,
    pub vrr: bool,
    /// The position of `self` with respect to its parent.
    pub relative: Option<Cardinal>,
    pub children: Vec<OutputNode>,
}

impl OutputNode {
    pub(crate) fn size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().fold(1, |acc, child| acc + child.size())
        }
    }
    /// Checks if the tree contains the given name.
    pub(crate) fn contains_name(&self, name: &str) -> bool {
        self.name == name ||
        self.children.iter().find(|node| node.contains_name(name)).is_some()
    }

    /// Adds `output` as a child of the output with `name`.
    pub(crate) fn add_child_of(&mut self, name: &str, card: Cardinal, output: Output) -> Result<(), Output> {
        // if we either have the name of the output we want to add,
        // or we don't have the name of the output we want to add `output` as a child to,
        // return error
        if self.contains_name(&output.name) || !self.contains_name(name) {
            return Err(output)
        }

        if self.name == name {
            /* If name shows up, add as child of ourselves */
            self.add_child(card, output);
        } else {
            /* Recursive call */
            self.children
                .iter_mut()
                .find(|node| node.contains_name(name))
                .map(|node| node.add_child_of(name, card, output))
                .expect("no node found with given name")?;
        }
        Ok(())
    }

    /// Adds `output` as a child of Self.
    fn add_child(&mut self, card: Cardinal, output: Output) {

        let delta = match card {
            Cardinal::Up | Cardinal::Down => self.position.size.height,
            Cardinal::Left | Cardinal::Right => self.position.size.width,
        };

        // anchor for the new output
        let pos = self.position.point.unidir_offset(delta, card);
        self.add_child_direct(pos, Some(card), output);
    }

    fn add_child_direct(&mut self, pos: Point<Physical>, card: Option<Cardinal>, output: Output) {
        let output_node = output.into_outputnode(pos, card);

        self.children.push(output_node);
    }
}

/// A platform-agnostic representation of a physical monitor, as managed by Toaru.
#[derive(Debug, Clone)]
pub struct Output {
    /// The name of the output, usually formatted `<connector>-<number>` (e.g. "eDP-2").
    pub name: String,
    /// Mode of the output
    pub mode: OutputMode,
    /// The scale of the output.
    pub scale: f32,
    /// Whether variable refresh-rate is enabled for this output.
    pub vrr: bool,
}

impl Output {
    pub(crate) fn into_outputnode(self, pos: Point<Physical>, dir: Option<Cardinal>) -> OutputNode {
        let Self {name, mode: OutputMode {size, refresh}, scale, vrr} = self;

        let position = Rectangle {
            point: pos,
            size
        };

        OutputNode {
            name, position, refresh, scale, vrr, 
            relative: dir,
            children: Vec::new()
        }
    }
}

/// A platform, agnostic representation of a physical monitor's mode, as managed by Toaru.
#[derive(Debug, Clone, Copy)]
pub struct OutputMode {
    /// The resolution of the mode.
    pub size: Size<Physical>,
    /// The refresh rate of the mode.
    pub refresh: i32,
}