use std::cell::Cell;

use tracing::debug;

use super::{
    update::{ResizeMain, Update, UpdateBorderPx},
    Layout, LayoutAction, LayoutCtxt, LayoutType,
};

use crate::core::Workspace;
use crate::types::{Cardinal, Geometry};
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
            main: Cell::new(None),
        }
    }
}

impl Layout for DynamicTiled {
    fn name(&self) -> &str {
        "DTiled"
    }

    fn layout(&self, ctxt: LayoutCtxt<'_>) -> Vec<LayoutAction> {
        self._layout(ctxt)
    }

    fn boxed(&self) -> Box<dyn Layout> {
        Box::new(self.clone())
    }

    fn receive_update(&self, update: &Update) {
        if let Some(ResizeMain(inc)) = update.as_update() {
            self.ratio.set(self.ratio.get() + inc);
        } else if let Some(UpdateBorderPx(new)) = update.as_update() {
            self.bwidth.set(*new);
        }
    }

    fn style(&self) -> super::LayoutType {
        LayoutType::Tiled
    }
}

#[doc(hidden)]
impl DynamicTiled {
    fn _layout(&self, ctxt: LayoutCtxt<'_>) -> Vec<LayoutAction> {
        let geom = ctxt.screen.effective_geom();
        let ws = ctxt.workspace;

        /* we have a main window */
        if let Some(main_id) = self.main.get() {
            self._layout_with_main(main_id, geom, ws)
        } else {
            // we have no main
            if ws.managed_count() == 0 {
                Vec::new()
            } else {
                // managed count >= 1
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

    fn _layout_with_main(
        &self,
        main_id: XWindowID,
        geom: Geometry,
        ws: &Workspace,
    ) -> Vec<LayoutAction> {
        use Cardinal::*;

        let bwidth = self.bwidth.get() as i32;

        /* weird ass bodge because of X server shenaniganery:
        we have to trim off double the bwidth because of how
        the X server counts window borders.*/
        let usable_geom = geom.trim(bwidth * 2, Right).trim(bwidth * 2, Down);

        if ws.managed_count() == 0 {
            /* managed count is 0 but we have a main,
            means the main just got closed and
            the workspace is now empty */
            debug!("Tiled count is 0, unsetting main");
            self.main.set(None);
            return vec![];
        }

        /* at this point we can assume managed count is >= 1 */

        // run check to set new main if needed
        if !ws.has_window_in_layout(main_id) {
            /* main window is no longer under layout,
            pick a new main */
            debug!("main is now off layout, choosing new main");
            let new_main = ws
                .clients_in_layout()
                .next()
                .expect("should have at least 1 client under layout")
                .id();

            self.main.set(Some(new_main));
        }
        let current_main = self.main.get().unwrap();

        // then proceed to generate geoms
        if ws.managed_count() == 1 {
            /* we only have a main window */
            debug!("Only main exists, tiling to full window");

            debug!("new window geom: {:?}", usable_geom);
            vec![LayoutAction::Resize {
                id: current_main,
                geom: usable_geom,
            }]
        } else {
            // managed count > 1
            debug_assert!(ws.managed_count() > 1);
            debug!("Multiple windows mapped, recalculating");

            let (main, sec) = usable_geom.split_vert_ratio(self.ratio.get());

            // do standard division and round up to nearest integer
            /* this ensures that if bwidth is odd, we always round up
            while keeping it unaffected if bwidth is even */
            let half_bwidth = (bwidth as f32 / 2.0).ceil() as i32;

            //let odd_bwidth = bwidth % 2 != 0;

            let mut ret = vec![LayoutAction::Resize {
                id: current_main,
                geom: main.trim(half_bwidth, Right),
            }];

            // get no of secondary windows
            let sec_count = if ws.managed_count() == 0 {
                unreachable!()
            } else {
                ws.managed_count() - 1
            };

            //todo: account for border width
            let sec_geoms = sec.split_horz_n(sec_count);

            ws.clients_in_layout()
                .filter(|c| c.id() != current_main)
                .enumerate()
                .for_each(|(i, c)| {
                    ret.push(LayoutAction::Resize {
                        id: c.id(),
                        geom: sec_geoms[i],
                    })
                });

            ret
        }
    }
}
