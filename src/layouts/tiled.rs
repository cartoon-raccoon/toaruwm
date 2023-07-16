use std::cell::Cell;

use tracing::debug;

use super::{
    Layout, LayoutType, LayoutAction, LayoutCtxt,
    update::{Update, ResizeMain}};

use crate::types::Geometry;
use crate::x::XWindowID;

/// A simple dynamic tiling layout, with a main window
/// and a stack on the side.
#[derive(Debug, Clone)]
pub struct DynamicTiled {
    // the proportion of space that the main window should take up
    ratio: Cell<f32>,
    // the border width set by the user.
    bwidth: Cell<u32>,
    // the ID of the main window, if set
    main: Cell<Option<XWindowID>>,
}

impl DynamicTiled {
    /// Creates a new DynamicTiled layout, with the given ratio
    /// and border width setup.
    pub fn new(ratio: f32, bwidth: u32) -> Self {
        Self {
            ratio: Cell::new(ratio),
            bwidth: Cell::new(bwidth),
            main: Cell::new(None)
        }
    }
}

impl Layout for DynamicTiled {
    fn name(&self) -> &str {
        "DTiled"
    }

    fn layout(&self, ctxt: LayoutCtxt<'_>) -> Vec<LayoutAction> {
        let geom = ctxt.screen.effective_geom();

        let ws = ctxt.workspace;

        /* we have a main window */
        if let Some(main_id) = self.main.get() {
            if ws.managed_count() == 0 {
                debug!("Tiled count is 0, unsetting main");
                self.main.set(None);
                vec![]
            } else if ws.managed_count() == 1 {
                debug!("Only main exists, tiling to full window");
                let new = Geometry::new(
                    0,
                    0,
                    geom.height - self.bwidth.get() as i32,
                    geom.width - self.bwidth.get() as i32,
                );
                vec![LayoutAction::Resize { id: main_id, geom: new }]
            } else {
                debug_assert!(ws.managed_count() > 1);
                debug!("Multiple windows mapped, recalculating");

                let (main, sec) = geom.split_vert_ratio(self.ratio.get());

                let mut ret = vec![LayoutAction::Resize { 
                    id: main_id, geom: main 
                }];

                // get no of secondary windows
                let sec_count = if ws.managed_count() == 0 {
                    0
                } else {
                    ws.managed_count() - 1
                };

                //todo: account for border width
                let sec_geoms = sec.split_horz_n(sec_count);

                ws.clients_in_layout()
                    .filter(|c| c.id() != main_id)
                    .enumerate()
                    .for_each(|(i, c)| {
                        ret.push(LayoutAction::Resize {
                            id: c.id(),
                            geom: sec_geoms[i],
                        })
                    });

                ret
            }
        } else { // we have no main
            if ws.managed_count() == 0 {
                Vec::new()
            } else { // managed count >= 1
                debug_assert!(ws.managed_count() >= 1);
                debug!("No main with at least one managed window");

                let main = ws.windows.get(0).unwrap();
                // set the main window
                self.main.set(Some(main.id()));
                /* now that we have a main window set,
                call back into ourselves, we should
                not recurse infinitely */
                self.layout(ctxt)
            }
        }
    }

    fn boxed(&self) -> Box<dyn Layout> {
        Box::new(self.clone())
    }

    fn receive_update(&self, update: &Update) {
        if let Some(ResizeMain(inc)) = update.as_update() {
            self.ratio.set(self.ratio.get() + inc)
        }
    }

    fn style(&self) -> super::LayoutType {
        LayoutType::Tiled
    }

}

