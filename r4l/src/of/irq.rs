// SPDX-License-Identifier: GPL-2.0

use crate::prelude::*;
use of::OfNode;

const MAX_PHANDLE_ARGS: usize = 32;
struct OfPhandleArgs {
    np: OfNode<'static>,
    args_count: usize,
    args: [u32; MAX_PHANDLE_ARGS],
}

impl OfPhandleArgs {
    fn of_irq_parse_one(node: OfNode<'static>, index: usize)
        -> Result<Self>  {
        let parent = node.interrupt_parent().ok_or(EINVAL)?;
        let intsize = parent.interrupt_cells().ok_or(EINVAL)?;
        
        pr_debug!("irq {} parent intsize={}\n",parent.compatible().unwrap().first(), intsize);

        let mut res = Self{np: parent, args_count: intsize, args: [0;MAX_PHANDLE_ARGS]};

        for i in 0..intsize {
            res.args[i] = of::of_property_read_u32(node, "interrupts", (index * intsize) + i).ok_or(EINVAL)?;
        }
        pr_debug!(" intspec={:?}\n", res.args);
        // TODO: Check if there are any interrupt-map translations to process 
        Ok(res)
    }
}

pub fn of_irq_get(node: OfNode<'static>, index: usize) -> Result<u32> {
    let oirq = OfPhandleArgs::of_irq_parse_one(node, index)?;
    if oirq.args_count != 3 {
        panic!("now only support arm interrupt")
    }
    Ok(oirq.args[1])
}
