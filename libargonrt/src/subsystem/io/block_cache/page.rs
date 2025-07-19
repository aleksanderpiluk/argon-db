use super::page_header::PageHeader;

pub struct Page(&PageHeader, *mut u8);

impl Clone for Page {
    fn clone(&self) -> Self {
        let header = self.0;
        header.ref_count_increment();

        Self(self.0, self.1);
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        let header = self.0;
        header.ref_count_decrement();
    }
}

// TODO: Access data via operator overload on Page
