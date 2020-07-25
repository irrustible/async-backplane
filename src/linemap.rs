use crate::{DeviceID, Line};

type Drain<'a, T> = std::vec::Drain<'a, T>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LineOp {
    Attach(Line),
    Detach(DeviceID),
}


#[derive(Debug)]
pub(crate) struct LineMap {
    inner: Inner,
}

#[derive(Debug)]
enum Inner {
    Small(Small),
    // Large,
}

#[derive(Debug, Default)]
struct Small {
    inner: Vec<(DeviceID, Option<Line>)>,
}

impl LineMap {

    pub fn new() -> Self { LineMap{ inner: Inner::Small(Small::default()) } }

    pub fn apply(&mut self, op: LineOp) {
        match op {
            LineOp::Attach(line) => { self.attach(line); }
            LineOp::Detach(did) => { self.detach(did); }
        }
    }

    /// Returns whether the line was found and overwritten.
    pub fn attach(&mut self, line: Line) -> bool {
        match self.inner {
            Inner::Small(ref mut small) => small.attach(line),
        }
    }

    /// Returns whether the item was found and deleted.
    pub fn detach(&mut self, did: DeviceID) -> bool {
        match self.inner {
            Inner::Small(ref mut small) => small.detach(did),
        }
    }

    pub fn drain(&mut self) -> Drain<(DeviceID, Option<Line>)> {
        match self.inner {
            Inner::Small(ref mut small) => small.inner.drain(..),
        }
    }

}

impl Small {

    fn attach(&mut self, line: Line) -> bool {
        let line_did = line.device_id();
        for (did, ref mut loon) in self.inner.iter_mut() {
            if *did == line_did {
                *loon = Some(line);
                return true;
            }
        }
        for (ref mut did, ref mut loon) in self.inner.iter_mut() {
            if loon.is_none() {
                *did = line_did;
                *loon = Some(line);
                return false;
            }
        }
        self.inner.push((line_did, Some(line)));
        return false;
    }

    fn detach(&mut self, did: DeviceID) -> bool {
        for (did2, ref mut loon) in self.inner.iter_mut() {
            if did == *did2 {
                *loon = None;
                return true;
            }
        }
        return false;
    }

}
