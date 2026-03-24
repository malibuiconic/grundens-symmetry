use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde::{Deserialize, Serialize};
use serde_json::json;

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/companyLocationCreate

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyAddressInput {
    pub address1: String,
    #[serde(rename = "address2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
    pub city: String,
    #[serde(rename = "provinceCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_code: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub zip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(rename = "firstName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyLocationPaymentTermsTemplateInput {
    #[serde(rename = "paymentTermsTemplateId")]
    pub payment_terms_template_id: Option<String>,  // Shopify GID for a payment terms template
    // If no template GID, Shopify also accepts NET terms inline — handled via note fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyLocationBuyerExperienceConfigInput {
    #[serde(rename = "checkoutToDraft")]
    pub checkout_to_draft: Option<bool>,
    #[serde(rename = "editOrderEnabled")]
    pub edit_order_enabled: Option<bool>,
    #[serde(rename = "payNowOnly")]
    pub pay_now_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyLocationInput {
    pub name: String,
    #[serde(rename = "externalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(rename = "billingAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<CompanyAddressInput>,
    #[serde(rename = "shippingAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<CompanyAddressInput>,
    /// Vec of tax exemption strings e.g. ["GOVERNMENT"]
    #[serde(rename = "taxExemptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exemptions: Option<Vec<String>>,
    #[serde(rename = "buyerExperienceConfiguration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_experience_configuration: Option<CompanyLocationBuyerExperienceConfigInput>,
}

/// Creates a company location and returns the new location GID.
/// `company_gid` = gid://shopify/Company/...
pub async fn company_location_create(
    shop: &ShopifyCreds,
    company_gid: &str,
    input: CompanyLocationInput,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation companyLocationCreate($companyId: ID!, $input: CompanyLocationInput!) {
            companyLocationCreate(companyId: $companyId, input: $input) {
                companyLocation {
                    id
                    name
                    externalId
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({
        "companyId": company_gid,
        "input": input,
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
        .get("companyLocationCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "companyLocationCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("companyLocationCreate")
        .and_then(|r| r.get("companyLocation"))
        .and_then(|cl| cl.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| {
            AppError::OtherError(
                "No companyLocation ID in companyLocationCreate response".to_string(),
            )
        })
}

/// Assigns payment terms to a company location using the paymentTermsCreate mutation.
/// Shopify does not support setting NET terms directly on companyLocationCreate;
/// it requires a separate paymentTermsCreate call on the location.
///
/// `location_gid`            = gid://shopify/CompanyLocation/...
/// `payment_terms_template_id` = gid://shopify/PaymentTermsTemplate/...
///   (query paymentTermsTemplates to find the right template GID for "NET 30" etc.)
pub async fn company_location_assign_payment_terms(
    shop: &ShopifyCreds,
    location_gid: &str,
    payment_terms_template_id: &str,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation paymentTermsCreate($referenceId: ID!, $paymentTermsAttributes: PaymentTermsCreateInput!) {
            paymentTermsCreate(
                referenceId: $referenceId,
                paymentTermsAttributes: $paymentTermsAttributes
            ) {
                paymentTerms {
                    id
                    paymentTermsName
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({
        "referenceId": location_gid,
        "paymentTermsAttributes": {
            "paymentTermsTemplateId": payment_terms_template_id,
        }
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
        .get("paymentTermsCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "paymentTermsCreate userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}
