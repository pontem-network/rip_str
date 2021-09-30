use crate::builder::NodeBuilder;

pub type Rope = Node;

#[derive(Clone)]
pub struct Node {
    pub(crate) height: usize,
    pub(crate) len: usize,
    pub(crate) inner: NodeVal,
}

#[derive(Clone)]
pub(crate) struct LeafUtf16 {
    pub val: Vec<char>,
}

#[derive(Clone)]
pub(crate) struct LeafL1 {
    pub val: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct LeafUnicode {
    pub val: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct Inode {
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

#[derive(Clone)]
pub(crate) enum NodeVal {
    LeafUtf(LeafUtf16),
    LeafL1(LeafL1),
    Internal(Inode),
}

impl From<&str> for Node {
    fn from(val: &str) -> Self {
        let mut builder = NodeBuilder::default();
        builder.push_str(val);
        builder.build()
    }
}
