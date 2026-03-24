use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use crate::{error::{AppError, UserError}, shopify_graphql_request_with_retries, RateLimiter, ShopifyCreds};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaObject {
    #[serde(rename="type")]
    pub definition_type: String,   // this matches definition handle!
    pub capabilities: Capabilities,
    pub handle: String, 
    pub fields: Vec<Field>         // these will match your definitions' keys
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaobjectField {
    pub key: String,              // for display name?
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Field {
    pub key: String,    
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capabilities {
    // #[serde(rename="onlineStore")]
    // pub online_store: Option<MetaobjectCapabilityOnlineStoreInput>,
    pub publishable: MetaobjectCapabilityDataPublishableInput
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaobjectCapabilityOnlineStoreInput {
    #[serde(rename="templateSuffix")]
    pub template_suffix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaobjectCapabilityDataPublishableInput {
    pub status: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MetaobjectStatus {
    ACTIVE,
    DRAFT,
}

impl MetaobjectStatus {
    pub fn from_str(role: &str) -> MetaobjectStatus {
        match role {
            "ACTIVE" => MetaobjectStatus::ACTIVE,              
            "DRAFT" | _=> MetaobjectStatus::DRAFT,
            
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetaobjectStatus::ACTIVE => String::from("ACTIVE"),
            MetaobjectStatus::DRAFT => String::from("DRAFT"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetaObject{
    metaobject: MetaObject
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetaObjectMutationResponse {
    #[serde(rename="metaobjectCreate")]
    pub metaobject_create: Option<NewMetaObjectResponse>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetaObjectResponse {
    pub metaobject: Option<NewMetaObjectInfo>,
    #[serde(rename="userErrors")]
    pub user_errors: Option<Vec<UserError>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetaObjectInfo {
    pub id: Option<String>,
    pub handle: Option<String>,
}

pub async fn create_metaobject(shop: &ShopifyCreds, meta_object: MetaObject, def_handle: &str, rate_limiter: &RateLimiter)
-> Result<NewMetaObjectInfo, AppError>{

   let handle = meta_object.handle.replace("-",""); 
   // println!("Handle: {}", handle);
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let mutation = format!("mutation
   CreateMetaobject($metaobject: MetaobjectCreateInput!){{
     metaobjectCreate(metaobject: $metaobject){{
        metaobject {{
            id
            handle
            {}: field(key: \"{}\"){{
                value
            }}
        }}
        userErrors {{
            field
            message
            code
        }}
     }}
   }}", handle, meta_object.handle);
   
  let new_metaobject = NewMetaObject { metaobject: meta_object };
    match shopify_graphql_request_with_retries::<NewMetaObjectMutationResponse, NewMetaObject>(gql_client, &mutation, Some(new_metaobject), 3, rate_limiter).await{
       Ok((response, extensions)) => {
          if let Some(new_metaobject_response) = response.metaobject_create {
             // println!("{:?}", new_metaobject_response.user_errors);
             if let Some(metaobject) = new_metaobject_response.metaobject {
                return Ok(metaobject)
             }
          }
          // if not a full response just send back empty response.
          return Ok(NewMetaObjectInfo {id: None, handle: None})
       },
       Err(e) => {
          Err(AppError::OtherError((e.to_string())))
       }
   } 
}