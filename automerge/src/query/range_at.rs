use crate::clock::Clock;
use crate::op_tree::{OpSetMetadata, OpTreeNode};
use crate::types::{Key, OpId};
use crate::{Prop, Value};
use std::fmt::Debug;
use std::ops::RangeBounds;

use super::VisWindow;

#[derive(Debug)]
pub(crate) struct RangeAt<'a, R: RangeBounds<Prop>> {
    clock: Clock,
    window: VisWindow,

    range: R,
    index: usize,
    /// number of visible elements seen.
    seen: usize,
    last_key: Option<Key>,

    index_back: usize,
    last_key_back: Option<Key>,

    root_child: &'a OpTreeNode,
    meta: &'a OpSetMetadata,
}

impl<'a, R: RangeBounds<Prop>> RangeAt<'a, R> {
    pub(crate) fn new(
        range: R,
        root_child: &'a OpTreeNode,
        meta: &'a OpSetMetadata,
        clock: Clock,
    ) -> Self {
        Self {
            clock,
            window: VisWindow::default(),
            range,
            index: 0,
            seen: 0,
            last_key: None,
            index_back: root_child.len(),
            last_key_back: None,
            root_child,
            meta,
        }
    }
}

impl<'a, R: RangeBounds<Prop>> Iterator for RangeAt<'a, R> {
    type Item = (Key, Value<'a>, OpId);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..self.index_back {
            let op = self.root_child.get(i)?;
            let visible = self.window.visible_at(op, i, &self.clock);
            self.index += 1;
            if Some(op.elemid_or_key()) != self.last_key && visible {
                self.last_key = Some(op.elemid_or_key());
                let contains = match op.key {
                    Key::Map(m) => self
                        .range
                        .contains(&Prop::Map(self.meta.props.get(m).clone())),
                    Key::Seq(_) => self.range.contains(&Prop::Seq(self.seen)),
                };
                self.seen += 1;
                if contains {
                    return Some((op.elemid_or_key(), op.value(), op.id));
                }
            }
        }
        None
    }
}

impl<'a, R: RangeBounds<Prop>> DoubleEndedIterator for RangeAt<'a, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        for i in (self.index..self.index_back).rev() {
            let op = self.root_child.get(i)?;
            self.index_back -= 1;
            if Some(op.elemid_or_key()) != self.last_key_back && op.visible() {
                self.last_key_back = Some(op.elemid_or_key());
                let contains = match op.key {
                    Key::Map(m) => self
                        .range
                        .contains(&Prop::Map(self.meta.props.get(m).clone())),
                    Key::Seq(_) => panic!("can't iterate through lists backwards"),
                };
                if contains {
                    return Some((op.elemid_or_key(), op.value(), op.id));
                }
            }
        }
        None
    }
}
