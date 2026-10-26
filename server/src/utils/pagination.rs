use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub struct Paginator {
    pub per_page: i64,
}
impl Paginator {
    pub const DEFAULT_PAGE_SIZE: i64 = 25;
    pub const MAX_PAGE_SIZE: i64 = 100;

    pub const fn new(page_size: Option<i64>) -> Self {
        let per_page = match page_size {
            None => Self::DEFAULT_PAGE_SIZE,
            Some(value) if value < 1 => 1,
            Some(value) if value > Self::MAX_PAGE_SIZE => Self::MAX_PAGE_SIZE,
            Some(value) => value,
        };
        Self { per_page }
    }
    pub const fn offset(self, page: u32) -> i64 {
        (page as i64).saturating_mul(self.per_page)
    }
    pub const fn page_size(self) -> i64 {
        self.per_page
    }
}

#[derive(Debug, Serialize)]
pub struct Page<Items> {
    pub items: Items,
    pub page: u32,
    pub per_page: i64,
    pub has_next: bool,
}

#[cfg(test)]
mod tests {
    use super::Paginator;

    #[test]
    fn calculates_zero_based_page_offsets() {
        let page_request = Paginator::new(None);
        assert_eq!(page_request.offset(0), 0);
        assert_eq!(page_request.offset(1), 25);
        assert_eq!(page_request.offset(3), 75);
        assert_eq!(page_request.offset(u32::MAX), i64::from(u32::MAX) * 25);
    }
}
