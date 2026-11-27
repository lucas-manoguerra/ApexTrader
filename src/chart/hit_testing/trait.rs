use crate::chart::hit_testing::result::{HitResult, HitTestContext};

pub trait HitTestable {
    fn hit_test(&self, ctx: &HitTestContext) -> Option<HitResult>;
}
