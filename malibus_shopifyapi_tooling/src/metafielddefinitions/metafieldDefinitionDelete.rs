use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::error::UserError;
use crate::AppError;
use crate::{RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries };

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DeleteMetafieldDefinitionResponse {
    #[serde(rename="metafieldDefinitionDelete")]
    metafield_definition_delete: Option<MetafieldDefinitionDelete>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MetafieldDefinitionDelete{
    #[serde(rename="deletedDefinitionId")]
    deleted_definition_id: Option<String>,
    #[serde(rename="userErrors")]
    user_errors: Option<Vec<UserError>>
}

pub async fn delete_metafield_definition(shop: &ShopifyCreds, rate_limiter: &RateLimiter, definition_gid: &str)
-> Result<(), AppError>{
   
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let mutation = r#"mutation
   DeleteMetafieldDefinition($id: ID!, $deleteAllAssociatedMetafields: Boolean!) {
    metafieldDefinitionDelete(id: $id, deleteAllAssociatedMetafields: $deleteAllAssociatedMetafields) {
       deletedDefinitionId
       userErrors {
          field
          message
          code
       }
    }
   }"#;
   let definition_to_delete = json!({ "id": definition_gid, "deleteAllAssociatedMetafields": true }); 
   match shopify_graphql_request_with_retries::<DeleteMetafieldDefinitionResponse, serde_json::Value>(gql_client, &mutation, Some(definition_to_delete), 3, rate_limiter).await{
      Ok((response, extensions)) => {
        if let Some(delete_definition) = response.metafield_definition_delete {
            if let Some(deleted_def_id) = delete_definition.deleted_definition_id {
                // println!("{}", deleted_def_id)
                return Ok(())
            }
            if let Some(user_errors) = delete_definition.user_errors {
                println!("{:?}", user_errors)
            }
        }
        return Ok(())
      },
      Err(e) => {
        Err(AppError::OtherError(e.to_string()))
      }
   }
}