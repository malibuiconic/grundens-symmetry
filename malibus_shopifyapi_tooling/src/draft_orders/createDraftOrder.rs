use crate::{ error::AppError, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use crate::error::UserError;
use serde_json::json;
use crate::draft_orders::DraftOrderInput;
use crate::{shopify_graphql_request_with_retries, RateLimiter};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Input {
    input: DraftOrderInput
}

#[derive(Debug, Clone, Deserialize)]
struct ShopifyDraftOrderCreateResponse {
    #[serde(rename="draftOrderCreate")]
    draft_order_create: Option<ShopifyDraftOrder>
}

#[derive(Debug, Clone, Deserialize)]
struct ShopifyDraftOrder {
    #[serde(rename="draftOrder")]
    draft_order: Option<ShopifyDraftOrderObj>
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShopifyDraftOrderObj {
    pub id: Option<String>,               // returned GID
    #[serde(rename="invoiceUrl")]
    pub invoice_url: Option<String>,      // Will be returned to Quote Obj on SF after

}

pub async fn create_draft_order(shop: &ShopifyCreds, draft_order_input: DraftOrderInput, rate_limiter: &RateLimiter)
-> Result<ShopifyDraftOrderObj, AppError>
{
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    draftOrderCreate($input: DraftOrderInput!){
        draftOrderCreate(input: $input){
            draftOrder{
                id
                invoiceUrl
            }
            userErrors {
                field
                message
            }    
        }
    }"#;
    let draft_order = Input { input: draft_order_input };
    match shopify_graphql_request_with_retries::<ShopifyDraftOrderCreateResponse, Input>(gql_client, &mutation, Some(draft_order), 3, rate_limiter).await{
         Ok((response, extenstions)) => {
            if let Some(draftorder_create) = &response.draft_order_create {
               if let Some(draftorder) = &draftorder_create.draft_order {
                  if let (Some(gid), Some(invoice_url)) = (&draftorder.id, &draftorder.invoice_url){
                      return Ok( ShopifyDraftOrderObj {
                          id: Some(gid.clone()),
                          invoice_url: Some(invoice_url.clone()),
                      })
                  }
               } 
            } 
            return Err(AppError::OtherError("Issue with Draftorder - Shopify".to_string()))
         },
         Err(e) => {
            Err(AppError::OtherError(e.to_string()))
         }
    }
}