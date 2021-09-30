use crate::node::{LeafL1, LeafUtf16, Node, NodeVal};
use std::cmp::min;

const MAX_BLOCK_SIZE: usize = 1024;
const MIN_BLOCK_SIZE: usize = 512;

#[derive(Default, Debug)]
pub struct NodeBuilder {}

impl NodeBuilder {
    fn push_node(&mut self, node: Node) {}

    pub fn push_str(&mut self, mut str: &str) {
        if str.len() <= MAX_BLOCK_SIZE {
            self.push_node(Self::make_mode(str));
        } else {
            while !str.is_empty() {
                let split_point = if str.len() > MAX_BLOCK_SIZE {
                    Self::find_split_point(str)
                } else {
                    str.len()
                };
                self.push_node(Self::make_mode(&str[..split_point]));
                str = &str[split_point..];
            }
        }
    }

    pub fn build(self) -> Node {
        todo!()
    }

    fn make_mode(str: &str) -> Node {
        if Self::is_l1(str) {
            Node {
                height: 0,
                len: str.len(),
                inner: NodeVal::LeafL1(LeafL1 {
                    val: str.as_bytes().to_vec(),
                }),
            }
        } else {
            Node {
                height: 0,
                len: str.len(),
                inner: NodeVal::LeafUtf(LeafUtf16 {
                    val: str.chars().collect(),
                }),
            }
        }
    }

    fn is_l1(str: &str) -> bool {
        for &b in str.as_bytes() {
            if b > 0x7F {
                return false;
            }
        }
        true
    }

    fn find_split_point(str: &str) -> usize {
        let mut split_point = 0;
        let mut is_utf16_start = false;

        for (i, b) in str.as_bytes().iter().enumerate() {
            split_point = i;
            if i < MIN_BLOCK_SIZE && !is_utf16_start && !b.is_ascii() {
                is_utf16_start = true;
            }

            if *b == b'\n' && i > MIN_BLOCK_SIZE {
                return split_point;
            }

            if i > MIN_BLOCK_SIZE && !b.is_ascii() && !is_utf16_start {
                return split_point;
            }

            if split_point >= MAX_BLOCK_SIZE {
                while !str.is_char_boundary(split_point) {
                    split_point -= 1;
                }
                return split_point;
            }
        }
        split_point
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::NodeBuilder;

    #[test]
    fn test_is_l1() {
        assert!(NodeBuilder::is_l1(""));
        assert!(NodeBuilder::is_l1("latin1"));
        assert!(NodeBuilder::is_l1("l"));
        assert!(NodeBuilder::is_l1("Hello world"));
        assert!(NodeBuilder::is_l1("!@$%^&!@$%^"));
        assert!(!NodeBuilder::is_l1("lé"));
        assert!(!NodeBuilder::is_l1("😈"));
    }

    fn split_point(str_1: &str, str_2: &str) {
        let val = format!("{}{}", str_1, str_2);
        let split_point = NodeBuilder::find_split_point(&val);
        assert_eq!(str_1, &val[..split_point]);
        assert_eq!(str_2, &val[split_point..]);
    }

    #[test]
    fn test_split_point() {
        split_point(
            "\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n\
This in the long string for testing\n",
            "\
This in the long string for testing\n\
This in the long string for testing\n",
        );
    }
}
