use itertools::Itertools;
use crate::context::QcContext;

impl QcContext {
    fn all_meta_iter(&self) -> () {
        Box::new(self.user_data.iter().map(|data| data.meta))
    }

    fn all_products_iter(&self) -> () {
        Box::new(self.user_data.keys().unique())
    }
}
