use fdt::node::NodeProperty;

use crate::find_phandle;
use crate::BigEndianU32;
use crate::OfNode;

const MAX_PHANDLE_ARGS: usize = 32;

pub struct OfPhandleArgs {
    pub np: OfNode<'static>,
    pub args_count: usize,
    pub args: [u32; MAX_PHANDLE_ARGS],
}

impl OfPhandleArgs {
    pub fn new(node: OfNode<'static>, args_count: usize, args: [u32; MAX_PHANDLE_ARGS]) -> Self {
        Self {
            np: node,
            args_count,
            args,
        }
    }
}

pub(crate) struct OfPhandleIterator {
    np: OfNode<'static>,
    cells_name: Option<&'static str>,
    cell_count: usize,
    lists: NodeProperty<'static>,
    lists_len: usize,
    curr_index: usize,
    curr_phandle: Option<OfNode<'static>>,
}

impl OfPhandleIterator {
    pub(crate) fn new(
        node: OfNode<'static>,
        list_name: &'static str,
        cells_name: Option<&'static str>,
        cell_count: usize,
    ) -> Option<Self> {
        let lists = node.property(list_name)?;
        let lists_len = lists.value.len(); //already 4 aligned
        Some(Self {
            np: node,
            cells_name,
            cell_count,
            lists,
            lists_len,
            curr_index: 0,
            curr_phandle: None,
        })
    }
}

impl Iterator for OfPhandleIterator {
    type Item = OfPhandleArgs;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_index >= self.lists_len {
            return None;
        }

        let phandle_val = BigEndianU32::from_bytes(&self.lists.value[self.curr_index..])
            .unwrap()
            .get();
        self.curr_phandle = Some(find_phandle(phandle_val).expect("Find Phandle"));
        // move cusor to phandle
        self.curr_index += 4;

        // parse cell count
        if let Some(name) = self.cells_name {
            let cell_count = self
                .curr_phandle
                .unwrap()
                .property(name)
                .expect("Cell name")
                .as_usize()
                .expect("Cell name");
            self.cell_count = cell_count;
        }

        // parse cell args
        let mut args = [0; MAX_PHANDLE_ARGS];
        for i in 0..self.cell_count {
            if self.curr_index >= self.lists_len {
                panic!("Cell count over lists len")
            }
            args[i] = BigEndianU32::from_bytes(&self.lists.value[self.curr_index..])
                .unwrap()
                .get();

            // move cusor
            self.curr_index += 4;
        }

        Some(OfPhandleArgs::new(self.np, self.cell_count, args))
    }
}
