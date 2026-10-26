//! The adaptive Huffman tree both directions keep in lockstep, and how its nodes are packed.

pub(crate) mod dynamic_huffman_tree;
pub(crate) mod node_type;
pub(crate) mod tree_entry;

pub(crate) use crate::tree::dynamic_huffman_tree::DynamicHuffmanTree;
pub(crate) use crate::tree::node_type::NodeType;
pub(crate) use crate::tree::tree_entry::TreeEntry;
