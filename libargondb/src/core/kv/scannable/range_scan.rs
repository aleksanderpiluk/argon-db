use crate::kv::primary_key::PrimaryKey;

pub enum RangeScanMarker {
    Start,
    End,
    Key(PrimaryKey<'static>),
}

pub struct RangeScanParams {
    pub from: RangeScanMarker,
    pub to: RangeScanMarker,
}

impl RangeScanParams {
    pub fn new(from: RangeScanMarker, to: RangeScanMarker) -> Self {
        Self { from, to }
    }

    pub fn full_range() -> Self {
        Self {
            from: RangeScanMarker::Start,
            to: RangeScanMarker::End,
        }
    }
}
