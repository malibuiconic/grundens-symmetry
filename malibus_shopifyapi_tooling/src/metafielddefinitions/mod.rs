#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use serde::de::{self, Visitor};
use std::fmt;
use serde_json::json;
use crate::{AppError, RateLimiter};
use crate::ShopifyCreds;
use crate::error::UserError;
use crate::metafields::MetaFieldType;
use crate::shopify_graphql_request_with_retries;

pub mod metafieldDefinitionDelete;   // if we need to delete metafieldDefinitions on bulk ;)

// https://shopify.dev/docs/api/admin-graphql/2024-04/mutations/metafieldDefinitionCreate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldDefinitionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<MetafieldAccessInput>,
    pub description: Option<String>,
    pub key: Option<String>,                   // required
    pub name: Option<String>,                  // required
    pub namespace: Option<String>,
    #[serde(rename="ownerType")]
    pub owner_type: Option<MetafieldOwnerType>, // required
    pub pin: Option<bool>,  //default:false
    #[serde(rename="type")]
    pub metafield_definition_type: Option<MetaFieldType>,    // required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="useAsCollectionCondition")]
    pub use_as_collection_condition: Option<bool>,  //default:false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validations: Option<Vec<MetafieldDefinitionValidationInput>>,     
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldDefinitionCreateResponse{
    #[serde(rename="metafieldDefinitionCreate")]
    pub metafield_definition_create: Option<CreatedDefinition>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreatedDefinition {
    #[serde(rename="createdDefinition")]
    pub created_definition: Option<Definition>,
    #[serde(rename="userErrors")]
    pub user_errors: Option<Vec<UserError>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Definition {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    pub namespace: Option<String>,
    #[serde(rename="ownerType")]
    pub owner_type: Option<MetafieldOwnerType>
}

impl MetafieldDefinitionInput {
    pub async fn metafield_definition_builder(shop: &ShopifyCreds, metafield_definition: MetafieldDefinitionInput, rate_limiter: &RateLimiter)
    -> Result<Definition, AppError>{
        
        let gql_client = ShopifyCreds::gql_client_builder(shop);
        let mutation = r#"mutation
        CreateMetafieldDefinition($definition: MetafieldDefinitionInput!){
            metafieldDefinitionCreate(definition: $definition){
                createdDefinition {
                    id
                    name
                    ownerType
                    key
                    namespace
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }"#;
        let definition = json!({"definition": metafield_definition});
        // println!("{}", serde_json::to_string_pretty(&definition).unwrap());
        match shopify_graphql_request_with_retries::<MetafieldDefinitionCreateResponse, serde_json::Value>(gql_client.clone(), &mutation, Some(definition), 3, rate_limiter).await{
            Ok((response, extensions)) => {
               // println!("{:?}", response);
               if let Some(created_metafield_definition) = response.metafield_definition_create {
                 if let Some(created_definition) = created_metafield_definition.created_definition {
                    // Newly Created MetafieldDefinition
                    return Ok(created_definition)
                 }                     
                 if let Some(user_errors) = created_metafield_definition.user_errors {
                    // Already Exists
                    println!("{:?}", user_errors)
                } 
              }
              // Already Exists
              return Ok( Definition {
                id: None,
                key: None,
                name: None,
                namespace: None,
                owner_type: None,
              })
            },
            Err(e) => {
              Err(AppError::OtherError(e.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldDefinitionValidationInput {
    pub name: Option<String>,   // required
    pub value: Option<String>,  // required
}

#[derive(Debug, Clone)]
pub enum MetafieldOwnerType {
    APIPERMISSION,
    ARTICLE,
    BLOG,
    CARTTRANSFORM,
    COLLECTION,
    COMPANY,
    COMPANYLOCATION,
    CUSTOMER,
    DELIVERYCUSTOMIZATION,
    DISCOUNT,
    DRAFTORDER,
    FULFILLMENTCONSTRAINTRULE,
    LOCATION,
    MARKET,
    MEDIAIMAGE,
    ORDER,
    ORDERROUTINGLOCATIONRULE,
    PAGE,
    PAYMENTCUSTOMIZATION,
    PRODUCT,
    PRODUCTVARIANT,
    SHOP,
    VALIDATION,
    PRODUCTIMAGE,
}

impl MetafieldOwnerType {
    pub fn from_str(role: &str) -> MetafieldOwnerType {
        match role {
            "API_PERMISSION" => MetafieldOwnerType::APIPERMISSION,
            "ARTICLE" => MetafieldOwnerType::ARTICLE,
            "BLOG" => MetafieldOwnerType::BLOG,
            "CARTTRANSFORM" => MetafieldOwnerType::CARTTRANSFORM,
            "COLLECTION" => MetafieldOwnerType::COLLECTION,
            "COMPANY" => MetafieldOwnerType::COMPANY,
            "COMPANY_LOCATION" => MetafieldOwnerType::COMPANYLOCATION,
            "CUSTOMER" => MetafieldOwnerType::CUSTOMER,
            "DELIVERY_CUSTOMIZATION" => MetafieldOwnerType::DELIVERYCUSTOMIZATION,
            "DISCOUNT" => MetafieldOwnerType::DISCOUNT,
            "DRAFTORDER" => MetafieldOwnerType::DRAFTORDER,
            "FULFILLMENT_CONSTRAINT_RULE" => MetafieldOwnerType::FULFILLMENTCONSTRAINTRULE,
            "LOCATION" => MetafieldOwnerType::LOCATION,
            "MARKET" => MetafieldOwnerType::MARKET,
            "MEDIA_IMAGE" => MetafieldOwnerType::MEDIAIMAGE,
            "ORDER" => MetafieldOwnerType::ORDER,
            "ORDER_ROUTING_LOCATION_RULE" => MetafieldOwnerType::ORDERROUTINGLOCATIONRULE,
            "PAGE" => MetafieldOwnerType::PAGE,
            "PAYMENT_CUSTOMIZATION" => MetafieldOwnerType::PAYMENTCUSTOMIZATION,
            "PRODUCT" | _=> MetafieldOwnerType::PRODUCT,
            "PRODUCTVARIANT" => MetafieldOwnerType::PRODUCTVARIANT,
            "SHOP" => MetafieldOwnerType::SHOP,
            "VALIDATION" => MetafieldOwnerType::VALIDATION,
            "PRODUCTIMAGE" => MetafieldOwnerType::PRODUCTIMAGE,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetafieldOwnerType::APIPERMISSION => String::from("API_PERMISSION"),
            MetafieldOwnerType::ARTICLE => String::from("ARTICLE"),
            MetafieldOwnerType::BLOG => String::from("BLOG"),
            MetafieldOwnerType::CARTTRANSFORM => String::from("CARTTRANSFORM"),
            MetafieldOwnerType::COLLECTION => String::from("COLLECTION"),
            MetafieldOwnerType::COMPANY => String::from("COMPANY"),
            MetafieldOwnerType::COMPANYLOCATION => String::from("COMPANY_LOCATION"),
            MetafieldOwnerType::CUSTOMER => String::from("CUSTOMER"),
            MetafieldOwnerType::DELIVERYCUSTOMIZATION => String::from("DELIVERY_CUSTOMIZATION"),
            MetafieldOwnerType::DISCOUNT => String::from("DISCOUNT"),
            MetafieldOwnerType::DRAFTORDER => String::from("DRAFTORDER"),
            MetafieldOwnerType::FULFILLMENTCONSTRAINTRULE => String::from("FULFILLMENT_CONSTRAINT_RULE"),
            MetafieldOwnerType::LOCATION => String::from("LOCATION"),
            MetafieldOwnerType::MARKET => String::from("MARKET"),
            MetafieldOwnerType::MEDIAIMAGE => String::from("MEDIA_IMAGE"),
            MetafieldOwnerType::ORDER => String::from("ORDER"),
            MetafieldOwnerType::ORDERROUTINGLOCATIONRULE => String::from("ORDER_ROUTING_LOCATION_RULE"),
            MetafieldOwnerType::PAGE => String::from("PAGE"),
            MetafieldOwnerType::PAYMENTCUSTOMIZATION => String::from("PAYMENT_CUSTOMIZATION"),
            MetafieldOwnerType::PRODUCT => String::from("PRODUCT"),
            MetafieldOwnerType::PRODUCTIMAGE => String::from("PRODUCTIMAGE"),
            MetafieldOwnerType::PRODUCTVARIANT => String::from("PRODUCTVARIANT"),
            MetafieldOwnerType::SHOP => String::from("SHOP"),
            MetafieldOwnerType::VALIDATION => String::from("VALIDATION"),
        }
    }
}

impl Serialize for MetafieldOwnerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MetafieldOwnerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaFieldTypeVisitor;

        impl<'de> Visitor<'de> for MetaFieldTypeVisitor {
            type Value = MetafieldOwnerType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta ownerType enum type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MetafieldOwnerType::from_str(value))
            }
        }

        deserializer.deserialize_str(MetaFieldTypeVisitor)
    }
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldAccessInput {
    pub admin: Option<MetafieldAdminAccess>,           // required
    pub grants: Option<Vec<MetafieldAccessGrantInput>>,
    pub storefront: Option<MetafieldStorefrontAccess>
}

#[derive(Debug, Clone)]
pub enum MetafieldStorefrontAccess {
    NONE,
    PUBLICREAD,
}

impl MetafieldStorefrontAccess {
    pub fn from_str(role: &str) -> MetafieldStorefrontAccess {
        match role {
            "NONE" => MetafieldStorefrontAccess::NONE,
            "PUBLIC_READ" | _=> MetafieldStorefrontAccess::PUBLICREAD,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetafieldStorefrontAccess::NONE => String::from("NONE"),
            MetafieldStorefrontAccess::PUBLICREAD => String::from("PUBLIC_READ"),
        }
    }
}

impl Serialize for MetafieldStorefrontAccess {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MetafieldStorefrontAccess {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaFieldTypeVisitor;

        impl<'de> Visitor<'de> for MetaFieldTypeVisitor {
            type Value = MetafieldStorefrontAccess;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta storefront access type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MetafieldStorefrontAccess::from_str(value))
            }
        }

        deserializer.deserialize_str(MetaFieldTypeVisitor)
    }
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldAccessGrantInput {
    pub access: Option<MetafieldGrantAccessLevel>,   // required
    pub grantee: Option<String>,                     // required
}

#[derive(Debug, Clone)]
pub enum MetafieldGrantAccessLevel {
    READ,
    READWRITE,
}

impl MetafieldGrantAccessLevel {
    pub fn from_str(role: &str) -> MetafieldGrantAccessLevel {
        match role {
            "READ" | _=> MetafieldGrantAccessLevel::READ,
            "READ_WRITE" => MetafieldGrantAccessLevel::READWRITE,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetafieldGrantAccessLevel::READ => String::from("READ"),
            MetafieldGrantAccessLevel::READWRITE => String::from("READ_WRITE"),
        }
    }
}

impl Serialize for MetafieldGrantAccessLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MetafieldGrantAccessLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaFieldTypeVisitor;

        impl<'de> Visitor<'de> for MetaFieldTypeVisitor {
            type Value = MetafieldGrantAccessLevel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta grant access level type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MetafieldGrantAccessLevel::from_str(value))
            }
        }

        deserializer.deserialize_str(MetaFieldTypeVisitor)
    }
}



#[derive(Debug, Clone)]
pub enum MetafieldAdminAccess {
    MERCHANTREAD,
    MERCHANTREADWRITE,
    PRIVATE,
    PUBLICREAD,
}

impl MetafieldAdminAccess {
    pub fn from_str(role: &str) -> MetafieldAdminAccess {
        match role {
            "MERCHANT_READ" => MetafieldAdminAccess::MERCHANTREAD,
            "MERCHANT_READ_WRITE" => MetafieldAdminAccess::MERCHANTREADWRITE,
            "PRIVATE" | _=> MetafieldAdminAccess::PRIVATE,
            "PUBLIC_READ" => MetafieldAdminAccess::PUBLICREAD,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetafieldAdminAccess::MERCHANTREAD => String::from("MERCHANT_READ"),
            MetafieldAdminAccess::MERCHANTREADWRITE => String::from("MERCHANT_READ_WRITE"),
            MetafieldAdminAccess::PRIVATE => String::from("PRIVATE"),
            MetafieldAdminAccess::PUBLICREAD => String::from("PUBLIC_READ")
        }
    }
}

impl Serialize for MetafieldAdminAccess {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MetafieldAdminAccess {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaFieldTypeVisitor;

        impl<'de> Visitor<'de> for MetaFieldTypeVisitor {
            type Value = MetafieldAdminAccess;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field admin access type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MetafieldAdminAccess::from_str(value))
            }
        }

        deserializer.deserialize_str(MetaFieldTypeVisitor)
    }
}

// Helper Functions
// Can include this above in struct as #[serde(with = "bool_to_string")]
mod bool_to_string {
    use serde::{self, Deserialize, Deserializer, Serializer};
    pub fn serialize<S>(x: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match x {
            Some(true) => s.serialize_some("true"),
            Some(false) => s.serialize_some("false"),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(d)?;
        match opt.as_deref() {
            Some("true") => Ok(Some(true)),
            Some("false") => Ok(Some(false)),
            None => Ok(None),
            _ => Err(serde::de::Error::custom("invalid value")),
        }
    }
}

// Examples: 
/*
let metafield_definition = MetafieldDefinitionInput {
    name: Some("Sequence".to_string()),
    access: None,
    description: Some("Collection Sequence".to_string()),
    key: Some("sequence".to_string()),
    namespace: Some("category".to_string()),
    owner_type: Some(MetafieldOwnerType::COLLECTION),
    pin: Some(true),
    metafield_definition_type: Some(MetaFieldType::SINGLELINETEXTFIELD),
    use_as_collection_condition: None,
    validations: None,
    };
    MetafieldDefinitionInput::metafield_definition_builder(&shop, metafield_definition).await;
*/