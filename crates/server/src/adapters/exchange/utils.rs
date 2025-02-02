use std::collections::HashSet;

use models::ProductSubscription;


pub fn get_products_to_subscibe(products_to_subscribe: &str) -> HashSet<ProductSubscription> {
    let mut products = HashSet::new();
    for product in products_to_subscribe.split(',') {
        products.insert(ProductSubscription {
            product_id: product.to_string(),
            subscribed: false,
        });
    }
    products
}
