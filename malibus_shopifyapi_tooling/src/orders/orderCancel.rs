use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde_json::json;

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/orderCancel

/// Cancels a Shopify order.
///
/// - `order_gid`       = gid://shopify/Order/...
/// - `reason`          = OrderCancelReason enum value as a string:
///                       "CUSTOMER" | "DECLINED" | "FRAUD" | "INVENTORY" | "OTHER" | "STAFF"
/// - `restock`         = true to restock line items back to inventory
/// - `refund`          = true to issue a refund (only for paid orders)
/// - `notify_customer` = whether to notify the customer via email
///
/// Returns `Ok(())` when the cancellation job is submitted. The actual cancellation
/// is asynchronous in Shopify — status can be tracked via the returned job ID.
pub async fn order_cancel(
    shop: &ShopifyCreds,
    order_gid: &str,
    reason: &str,
    restock: bool,
    refund: bool,
    notify_customer: bool,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation orderCancel(
            $orderId: ID!,
            $reason: OrderCancelReason!,
            $restock: Boolean!,
            $refund: Boolean!,
            $notifyCustomer: Boolean
        ) {
            orderCancel(
                orderId: $orderId,
                reason: $reason,
                restock: $restock,
                refund: $refund,
                notifyCustomer: $notifyCustomer
            ) {
                job {
                    id
                    done
                }
                orderCancelUserErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({
        "orderId": order_gid,
        "reason": reason,
        "restock": restock,
        "refund": refund,
        "notifyCustomer": notify_customer,
    });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("orderCancel")
        .and_then(|r| r.get("orderCancelUserErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "orderCancel userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}
