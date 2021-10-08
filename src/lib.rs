use crate::segment::Segment;
use crate::splitter::Splitter;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::Range;

pub(crate) mod segment;
pub(crate) mod splitter;

#[derive(Debug)]
pub struct RipString {
    nodes: Vec<Box<Segment>>,
    /// Index of last edit node.
    last_edit: usize,
}

impl RipString {
    pub fn new() -> RipString {
        let seq = Segment::default();
        RipString {
            nodes: vec![Box::new(seq)],
            last_edit: 0,
        }
    }

    pub fn edit(&mut self, range: Range<usize>, new: &str) {
        if range.len() == 0 {
            if new.is_empty() {
                return;
            }

            let seg_index = self.find_segment(range.start);
            let node = &mut self.nodes[seg_index];
            if let Some(new_nodes) = node.insert(range.start, new) {
                if seg_index == self.nodes.len() - 1 {
                    self.nodes.extend(new_nodes);
                } else {
                    let suffix = self.nodes.split_off(seg_index + 1);
                    self.nodes.extend(new_nodes);
                    self.nodes.extend(suffix);
                }
            }
            self.last_edit = seg_index;
            self.fix_index_from(seg_index);
        } else {
            if new.is_empty() {
                let seg_index = self.find_segment(range.start);
                let last_seg_index = self.find_segment(range.end);

                if last_seg_index == seg_index {
                    let node = &mut self.nodes[seg_index];
                    if let Some(node) = node.cut(range) {
                        if seg_index == self.nodes.len() - 1 {
                            self.nodes.push(node);
                        } else {
                            self.nodes.insert(seg_index + 1, node);
                        }
                    }
                } else {
                    let node = &mut self.nodes[seg_index];
                    // We ignore the result as in this case, it is always None.
                    node.cut(range.clone());
                    let node = &mut self.nodes[last_seg_index];
                    if let Some(node) = node.cut(node.index()..range.end) {
                        self.nodes[last_seg_index] = node;
                    }
                    let mut new_nodes = Vec::with_capacity(self.nodes.len());
                    mem::swap(&mut new_nodes, &mut self.nodes);
                    self.nodes.extend(
                        new_nodes
                            .into_iter()
                            .enumerate()
                            .filter(|(i, _n)| *i <= seg_index || *i >= last_seg_index)
                            .map(|(_, b)| b),
                    );
                }
                self.last_edit = last_seg_index;
                self.fix_index_from(seg_index);
            } else {
                let seg_index = self.find_segment(range.start);
                let last_seg_index = self.find_segment(range.end);

                //
                // if last_seg_index == seg_index {
                //     let node = &mut self.nodes[seg_index];
                //     if let Some(node) = node.replace(range, new) {
                //         if seg_index == self.nodes.len() - 1 {
                //             self.nodes.push(node);
                //         } else {
                //             self.nodes.insert(seg_index + 1, node);
                //         }
                //     }
                // } else {
                //     let node = &mut self.nodes[seg_index];
                //     // We ignore the result as in this case, it is always None.
                //     node.cut(range.clone());
                //     let node = &mut self.nodes[last_seg_index];
                //     if let Some(node) = node.cut(node.index()..range.end) {
                //         self.nodes[last_seg_index] = node;
                //     }
                //     let mut new_nodes = Vec::with_capacity(self.nodes.len());
                //     mem::swap(&mut new_nodes, &mut self.nodes);
                //     self.nodes.extend(
                //         new_nodes
                //             .into_iter()
                //             .enumerate()
                //             .filter(|(i, _n)| *i <= seg_index || *i >= last_seg_index)
                //             .map(|(_, b)| b),
                //     );
                // }
                //
                //
                //
                //
                //
                //
                // let seg_index = self.find_segment(range.start);
                // let last_seg_index = self.find_segment(range.end);
                //
                // let node = &mut self.nodes[seg_index];
                // if let Some(new_nodes) = node.replace(range, new) {
                //     if seg_index == self.nodes.len() - 1 {
                //         self.nodes.extend(new_nodes);
                //     } else {
                //         let suffix = self.nodes.split_off(seg_index + 1);
                //         self.nodes.extend(new_nodes);
                //         self.nodes.extend(suffix);
                //     }
                // }
                //
                // self.last_edit = last_seg_index;
                // self.fix_index_from(seg_index);
            }
        }
    }

    fn fix_index_from(&mut self, seg_index: usize) {
        let last_right_node = &self.nodes[seg_index];
        let mut next_index = last_right_node.index() + last_right_node.len();
        for i in seg_index + 1..self.nodes.len() {
            self.nodes[i].set_index(next_index);
            next_index += self.nodes[i].len();
        }
    }

    fn find_segment(&self, index: usize) -> usize {
        if self.nodes[self.last_edit].contains(index) {
            return self.last_edit;
        }

        self.nodes
            .binary_search_by(|seg| seg.ord(index))
            .expect("Index is out of bound")
    }
}

impl From<String> for RipString {
    fn from(val: String) -> Self {
        let (_, mut nodes) = Splitter::new(&val).fold((0, vec![]), |(mut index, mut acc), seg| {
            index += seg.len();
            let seg = Segment::new(index, seg);
            acc.push(Box::new(seg));
            (index, acc)
        });

        if nodes.is_empty() {
            nodes.push(Box::new(Segment::default()));
        }

        RipString {
            nodes,
            last_edit: 0,
        }
    }
}

impl Display for RipString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            node.fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::RipString;

    #[test]
    pub fn edit_test() {
        let mut rip_str = RipString::new();
        rip_str.edit(0..0, "H");
        rip_str.edit(1..1, "e");
        rip_str.edit(2..2, "l");
        rip_str.edit(3..3, "l");
        rip_str.edit(4..4, "o");
        rip_str.edit(5..5, " ");
        rip_str.edit(6..6, "world");
        rip_str.edit(11..11, ". ");
        assert_eq!(rip_str.to_string(), "Hello world. ".to_string());
        rip_str.edit(13..13, "Привет мир.");
        assert_eq!(rip_str.to_string(), "Hello world. Привет мир.".to_string());
        rip_str.edit(13..20, "");
        assert_eq!(rip_str.to_string(), "Hello world. мир.".to_string());
        rip_str.edit(11..13, "");
        assert_eq!(rip_str.to_string(), "Hello worldмир.".to_string());
        rip_str.edit(11..11, ". Привет ");
        assert_eq!(rip_str.to_string(), "Hello world. Привет мир.".to_string());
        dbg!(&rip_str);
        rip_str.edit(6..20, " ");
        assert_eq!(rip_str.to_string(), "Hello мир.".to_string());
    }
}
