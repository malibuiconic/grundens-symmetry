use crate::metafields::MetaFieldType;
use crate::shopify_graphql_request_with_retries;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::AppError;
use crate::error::UserError;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaObjectDefinition {
    pub definition: Definition              // required
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Definition {
    #[serde(rename="type")]
    pub meta_type: String,        // required (3-255 alphanumeric, hyphen, underscore)
    pub name: Option<String>,
    pub access: Option<Access>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="displayNameKey")]
    pub display_name_key: Option<String>,
    pub capabilities: Option<Capabilities>,
    pub description: Option<String>,
    #[serde(rename="fieldDefinitions")]
    pub field_definitions: Vec<FieldDefinition>   // required
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldDefinition {
    pub name: Option<String>,
    pub key: String,               // required - can't be changed 3-64 chars long
    #[serde(rename="type")]
    pub field_type: String,        // required - metafield type (ie. single_line_text_field)
    pub validations: Option<Vec<MetafieldDefininitionValidationInput>>  // required to reference other metaobjects!
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldDefininitionValidationInput {
    pub name: Option<String>,  // non-null
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Validation {
    pub name: Option<String>, // non-null
    #[serde(rename="type")]
    pub validation_type: Option<String>,  // non-null
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capabilities {
    #[serde(rename="onlineStore")]
    pub online_store: Option<OnlineStore>,
    pub renderable: Option<Renderable>,
    pub translatable: Option<Translatable>,
    pub publishable: Option<Publishable>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Publishable {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Translatable {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Renderable {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<MetaobjectCapabilityDefinitionDataRenderableInput>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaobjectCapabilityDefinitionDataRenderableInput {
    #[serde(rename="metaDescriptionKey")]
    pub meta_description_key: Option<String>,
    #[serde(rename="metaTitleKey")]
    pub meta_title_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnlineStore {
    pub enabled: bool,
    pub data: Option<URLHANDLE>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct URLHANDLE {
    #[serde(rename="urlHandle")]
    pub url_handle: Option<String>         // "/somehandle"     
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Access {
    pub admin: Option<String>,             // Type ENUM ADMIN
    pub storefront: Option<String>,        // Type ENUM STOREFRONT
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum STOREFRONT {
    NONE,
    PUBLICREAD,
}

impl STOREFRONT {
    pub fn from_str(role: &str) -> STOREFRONT {
        match role {
            "NONE" => STOREFRONT::NONE,              
            "PUBLIC_READ" | _=> STOREFRONT::PUBLICREAD, 
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            STOREFRONT::NONE => String::from("NONE"),
            STOREFRONT::PUBLICREAD => String::from("PUBLIC_READ"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ADMIN {
    MERCHANTREAD,
    MERCHANTREADWRITE,
    PRIVATE,
    PUBLICREAD,
    PUBLICREADWRITE,
}

impl ADMIN {
    pub fn from_str(role: &str) -> ADMIN {
        match role {
            "MERCHANT_READ" => ADMIN::MERCHANTREAD,              
            "MERCHANT_READ_WRITE" => ADMIN::MERCHANTREADWRITE,
            "PRIVATE" | _=> ADMIN::PRIVATE,
            "PUBLIC_READ" => ADMIN::PUBLICREAD,
            "PUBLIC_READ_WRITE" => ADMIN::PUBLICREADWRITE
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            ADMIN::MERCHANTREAD => String::from("MERCHANT_READ"),
            ADMIN::MERCHANTREADWRITE => String::from("MERCHANT_READ_WRITE"),
            ADMIN::PRIVATE => String::from("PRIVATE"),
            ADMIN::PUBLICREAD => String::from("PUBLIC_READ"),
            ADMIN::PUBLICREADWRITE => String::from("PUBLIC_READ_WRITE"),
        }
    }
}

// RESPONSE
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateMetaObjDefResponse {
    #[serde(rename="metaobjectDefinitionCreate")]
    pub metaobjectdefinitioncreate: Option<MetaObjectDefinitionCreate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaObjectDefinitionCreate {
    #[serde(rename="metaobjectDefinition")]
    pub metaobject_definition: Option<NewMetaObjectDefinition>,
    #[serde(rename="userErrors")]
    pub user_errors: Option<Vec<UserError>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetaObjectDefinition {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename="type")]
    pub def_type: Option<String>,
}

// Metadefinition Creator 
pub async fn create_metaobject_definition(shop: &ShopifyCreds, metaobject_definition: MetaObjectDefinition, rate_limiter: &RateLimiter)
-> Result<String, AppError>{
   
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    metaobjectDefinitionCreate($definition: MetaobjectDefinitionCreateInput!) {
        metaobjectDefinitionCreate(definition: $definition) {
            metaobjectDefinition {
               id
               name
               type
            }
            userErrors {
               field
               message
               code
            }
        }
    }
    "#;
    match shopify_graphql_request_with_retries::<CreateMetaObjDefResponse, MetaObjectDefinition>(gql_client, &mutation, Some(metaobject_definition), 3, rate_limiter).await{
        Ok((success,extensions)) => {
            // if a new_gid!
            if let Some(mobj_def_create) = success.clone().metaobjectdefinitioncreate {
               if let Some(new_definition) = mobj_def_create.metaobject_definition {
                  if let Some(gid) = new_definition.id {
                     //println!("{}", gid);
                     return Ok(gid)
                  }
               }
            }
            // if already taken? 
            if let Some(mobj_def_create) = success.metaobjectdefinitioncreate {
                if let Some(user_errors) = mobj_def_create.user_errors {
                    if let Some(code) = user_errors[0].code.clone() {
                      return Ok(code) // will return TAKEN
                    }
                }
            }
            Err(AppError::MetaObjectDefinitionCreateError(anyhow::Error::msg("")))
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}

