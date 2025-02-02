
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ProductSubscription {
    pub product_id: String,
    pub subscribed: bool,
}
