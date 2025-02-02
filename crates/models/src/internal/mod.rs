
mod product;
mod order_book;
mod message;

pub use product::*;
pub use order_book::*;
pub use message::*;



#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ProductSubscription {
    pub product_id: String,
    pub subscribed: bool,
}
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Exchange {
    Okex,
    Deribit,
}
