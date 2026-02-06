mod range_scan;
mod row_scan;
mod set_scan;

use async_trait::async_trait;
pub use range_scan::{RangeScanMarker, RangeScanParams};
pub use row_scan::RowScanParams;

#[async_trait]
pub trait Scannable {
    // async fn full_scan(&self) -> Result<(), ()> {
    //     self.range_scan(RangeScanParams::full_range()).await
    // }

    async fn range_scan(&self, params: RangeScanParams) -> Result<(), ()>;

    async fn row_scan(&self, params: RowScanParams) -> Result<(), ()>;
}
