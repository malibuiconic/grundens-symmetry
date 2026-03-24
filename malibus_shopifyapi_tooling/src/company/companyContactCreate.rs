use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────────────────────────────────────────
// companyContactCreate
// Associates an existing Shopify customer as a contact of a B2B company.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/companyContactCreate
// ──────────────────────────────────────────────────────────────────────────────

/// Associates a customer with a company as a contact.
/// Returns the new company contact GID.
///
/// `company_gid`  = gid://shopify/Company/...
/// `customer_gid` = gid://shopify/Customer/...
/// `title`        = optional job title e.g. "Director of Purchasing"
pub async fn company_contact_create(
    shop: &ShopifyCreds,
    company_gid: &str,
    customer_gid: &str,
    title: Option<&str>,
    locale: Option<&str>,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation companyContactCreate($companyId: ID!, $input: CompanyContactInput!) {
            companyContactCreate(companyId: $companyId, input: $input) {
                companyContact {
                    id
                    isMainContact
                    customer {
                        id
                        email
                    }
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let mut contact_input = json!({ "customerId": customer_gid });
    if let Some(t) = title {
        contact_input["title"] = json!(t);
    }
    if let Some(l) = locale {
        contact_input["locale"] = json!(l);
    }

    let variables = json!({
        "companyId": company_gid,
        "input": contact_input,
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
        .get("companyContactCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "companyContactCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("companyContactCreate")
        .and_then(|r| r.get("companyContact"))
        .and_then(|cc| cc.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| {
            AppError::OtherError(
                "No companyContact ID in companyContactCreate response".to_string(),
            )
        })
}

// ──────────────────────────────────────────────────────────────────────────────
// companyContactRoleAssign
// Grants a contact a specific role at a company location.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/companyContactRoleAssign
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyContactRoleAssignment {
    /// Role name: "ORDERING" | "ACCOUNT_MANAGER" etc.
    #[serde(rename = "companyContactRoleId")]
    pub company_contact_role_id: Option<String>,
    #[serde(rename = "companyLocationId")]
    pub company_location_id: String,
}

/// Assigns roles to a company contact at one or more locations.
///
/// NOTE: Shopify requires the `companyContactRoleId` (GID), not just the role name string.
/// You'll need to query `companyContactRoles` first to get the GIDs for "ORDERING" etc.,
/// or store them in config after a one-time lookup.
pub async fn company_contact_assign_roles(
    shop: &ShopifyCreds,
    company_contact_gid: &str,
    roles: Vec<CompanyContactRoleAssignment>,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation companyContactRoleAssign(
            $companyContactId: ID!,
            $rolesToAssign: [CompanyContactRoleAssignment!]!
        ) {
            companyContactRoleAssign(
                companyContactId: $companyContactId,
                rolesToAssign: $rolesToAssign
            ) {
                assignedRoles {
                    companyContactRole {
                        id
                        name
                    }
                    companyLocation {
                        id
                        name
                    }
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
        "companyContactId": company_contact_gid,
        "rolesToAssign": roles,
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
        .get("companyContactRoleAssign")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "companyContactRoleAssign userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}
