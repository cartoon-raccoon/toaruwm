use tracing::{debug, warn};

use super::LayoutAction::{self, *};

use crate::core::{Screen, Workspace};
use crate::types::Geometry;

// Ws and scr are only included to satisfy the function signature.
pub(super) fn gen_layout(
    ws: &Workspace,
    scr: &Screen,
    bwidth: u32,
    ratio: f32,
) -> Vec<LayoutAction> {
    let geom = scr.effective_geom();
    if let Some(id) = ws.master() {
        if ws.tiled_count() == 0 {
            warn!("Tiled count is 0, but ws has master set");
            return vec![UnsetMaster];
        }
        if ws.tiled_count() == 1 {
            debug!("Only master exists, tiling to full window");
            let new = Geometry::new(
                0,
                0,
                geom.height - bwidth as i32,
                geom.width - bwidth as i32,
            );
            vec![SetMaster(id), Resize { id, geom: new }]
        } else {
            debug!("Multiple windows mapped, recalculating");

            let (master, slaves) = geom.split_vert_ratio(ratio);

            let mut ret = vec![SetMaster(id), Resize { id, geom: master }];

            // get no of slave windows
            let slave_count = if ws.tiled_count() == 0 {
                0
            } else {
                ws.tiled_count() - 1
            };

            //todo: account for border width
            let slave_geoms = slaves.split_horz_n(slave_count);

            ws.clients()
                .filter(|c| c.id() != id && c.is_tiled())
                .enumerate()
                .for_each(|(i, c)| {
                    ret.push(Resize {
                        id: c.id(),
                        geom: slave_geoms[i],
                    })
                });

            ret
        }
    } else {
        if ws.tiled_count() == 1 {
            debug!("Only master exists, tiling to full window");
            let id = ws.windows.get(0).unwrap().id();
            let new = Geometry::new(
                0,
                0,
                geom.height - bwidth as i32,
                geom.width - bwidth as i32,
            );
            return vec![SetMaster(id), Resize { id, geom: new }];
        }
        Vec::new()
    }
}
