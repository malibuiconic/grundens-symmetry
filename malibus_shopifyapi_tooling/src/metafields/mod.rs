use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use std::fmt::Debug;
use serde::de::{self, Visitor};
use std::fmt;
use chrono::prelude::*;
use crate::error::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::metafielddefinitions::MetafieldOwnerType;
use crate::PageInfo;
use crate::shopify_graphql_request_with_retries;

//https://shopify.dev/docs/apps/custom-data/metafields/types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metafields{
    pub metafields: Option<Vec<MetafieldInput>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub key: Option<String>,
    pub namespace: Option<String>,
    #[serde(rename="ownerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_gid: Option<String>,
    #[serde(rename="type")]
    pub meta_type: Option<MetaFieldType>,  
    pub value: Option<MetaFieldValue>,
}

impl MetafieldInput {
    pub async fn metafield_builder(shop: &ShopifyCreds, metafield_input: MetafieldInput, rate_limiter: &RateLimiter) -> Result<String, AppError> {
        let gql_client = ShopifyCreds::gql_client_builder(shop);
        let mutation = format!("mutation
            MetafieldsSet($metafields: [MetafieldsSetInput!]!) {{
                metafieldsSet(metafields: $metafields) {{
                    metafields {{
                        id
                        key
                        namespace
                        value
                        createdAt
                        updatedAt
                    }}
                    userErrors {{
                        field
                        message
                        code
                    }}
                }}
            }}
        ");
        let metafields = Metafields {
            metafields: Some(vec![metafield_input.clone()]) 
        };
        // println!("{}\n", serde_json::to_string_pretty(&metafields).unwrap());
        match shopify_graphql_request_with_retries::<MetafieldCreateResponse, Metafields>(gql_client.clone(), &mutation, Some(metafields), 3, rate_limiter).await{
            Ok((response, extensions)) => {
               // println!("{:?}", response);
               if let Some(metafields_set) = response.metafields_set {
                  if let Some(metafields) = metafields_set.metafields {
                     for (idx, metafield) in metafields.iter().enumerate() {
                        if idx == 0 {
                            if let Some(gid) = &metafield.id {
                                return Ok(gid.to_string())
                            }
                        }
                     }
                  }
               }
               let err_msg = "No Metafield Created!";
               Err(AppError::OtherError(err_msg.to_string()))
            },
            Err(e) => {
                // println!("{}", e);
                Err(AppError::OtherError(e.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)] // This attribute allows serialization/deserialization without tagging the type
pub enum MetaFieldValue {
    // NUMBERINTEGER(u32),
    NUMBERDECIMAL(f64),
    SINGLELINETEXTFIELD(String), //The data stored in the metafield. Always stored as a string, regardless of the metafield's type.
    JSON(serde_json::Value),
    // BOOLEAN(bool),
    // METAOBJECTREFERENCE(String),  
    // Add other types as needed (based off metafield types below)
}


#[derive(Debug, Clone)]
pub enum MetaFieldType {
    // Supported Types
    BOOLEAN,
    COLOR,
    DATE,
    DATETIME,
    DIMENSION,
    JSON,
    LINK,
    MONEY,
    MULTILINETEXTFIELD,
    NUMBERDECIMAL,
    NUMBERINTEGER,
    RATING,
    RICHTEXTFIELD,
    SINGLELINETEXTFIELD,
    URL,
    VOLUME,
    WEIGHT,
    // Reference Types
    COLLECTIONREFERENCE,
    FILEREFERENCE,
    METAOBJECTREFERENCE,
    MIXEDREFERENCE,
    PAGEREFERENCE,
    PRODUCTREFERENCE,
    VARIANTREFERENCE,
    // List Types
    LISTCOLLECTIONREFERENCE,
    LISTCOLOR,
    LISTDATE,
    LISTDATETIME,
    LISTDIMENSION,
    LISTFILEREFERENCE,
    LISTLINK,
    LISTMETAOBJECTREFERENCE,
    LISTMIXEDREFERENCE,
    LISTNUMBERINTEGER,
    LISTNUMBERDECIMAL,
    LISTPAGEREFERENCE,
    LISTPRODUCTREFERENCE,
    LISTRATING,
    LISTSINGLELINETEXTFIELD,
    LISTURL,
    LISTVARIANTREFERENCE,
    LISTVOLUME,
    LISTWEIGHT,
}

impl MetaFieldType {
    pub fn from_str(role: &str) ->  MetaFieldType {
        match role {
            "boolean" =>  MetaFieldType::BOOLEAN,              
            "color" =>  MetaFieldType::COLOR,
            "date" =>  MetaFieldType::DATE,
            "date_time" =>  MetaFieldType::DATETIME,
            "dimension" =>  MetaFieldType::DIMENSION,
            "json" =>  MetaFieldType::JSON,
            "link" => MetaFieldType::LINK,
            "money" =>  MetaFieldType::MONEY,
            "multi_line_text_field" =>  MetaFieldType::MULTILINETEXTFIELD,
            "number_decimal" =>  MetaFieldType::NUMBERDECIMAL,
            "number_integer" =>  MetaFieldType::NUMBERINTEGER,
            "rating" =>  MetaFieldType::RATING,
            "rich_text_field" =>  MetaFieldType::RICHTEXTFIELD,
            "single_line_text_field" | _=>  MetaFieldType::SINGLELINETEXTFIELD,
            "url" =>  MetaFieldType::URL,
            "volume" =>  MetaFieldType::VOLUME,
            "weight" =>  MetaFieldType::WEIGHT,
            "collection_reference" => MetaFieldType::COLLECTIONREFERENCE,
            "file_reference" => MetaFieldType::FILEREFERENCE,
            "metaobject_reference" => MetaFieldType::METAOBJECTREFERENCE,
            "mixed_reference" => MetaFieldType::MIXEDREFERENCE,
            "page_reference" => MetaFieldType::PAGEREFERENCE,
            "product_reference" => MetaFieldType::PRODUCTREFERENCE,
            "variant_reference" => MetaFieldType::VARIANTREFERENCE,
            // LIST TYPES
            "list.collection_reference" => MetaFieldType::LISTCOLLECTIONREFERENCE,
            "list.color" => MetaFieldType::LISTCOLOR,
            "list.date" => MetaFieldType::LISTDATE,
            "list.date_time" => MetaFieldType::LISTDATETIME,
            "list.dimension" => MetaFieldType::LISTDIMENSION,
            "list.file_reference" => MetaFieldType::LISTFILEREFERENCE,
            "list.link" => MetaFieldType::LISTLINK,
            "list.metaobject_reference" => MetaFieldType::LISTMETAOBJECTREFERENCE,
            "list.mixed_reference" => MetaFieldType::LISTMIXEDREFERENCE,
            "list.number_integer" => MetaFieldType::LISTNUMBERINTEGER,
            "list.number_decimal" => MetaFieldType::LISTNUMBERDECIMAL,
            "list.page_reference" => MetaFieldType::LISTPAGEREFERENCE,
            "list.product_reference" => MetaFieldType::LISTPRODUCTREFERENCE,
            "list.single_line_text_field" => MetaFieldType::LISTSINGLELINETEXTFIELD,
            "list.url" => MetaFieldType::LISTURL,
            "list.variant_reference" => MetaFieldType::LISTVARIANTREFERENCE,
            "list.volume" => MetaFieldType::LISTVOLUME,
            "list.weight" => MetaFieldType::LISTWEIGHT,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            MetaFieldType::BOOLEAN => String::from("boolean"),             
            MetaFieldType::COLOR => String::from("color"),
            MetaFieldType::DATE => String::from("date"),
            MetaFieldType::DATETIME => String::from("date_time"),
            MetaFieldType::DIMENSION => String::from("dimension"),
            MetaFieldType::JSON => String::from("json"),
            MetaFieldType::LINK => String::from("link"),
            MetaFieldType::MONEY => String::from("money"),
            MetaFieldType::MULTILINETEXTFIELD => String::from("multi_line_text_field"),
            MetaFieldType::NUMBERDECIMAL => String::from("number_decimal"),
            MetaFieldType::NUMBERINTEGER => String::from("number_integer"),
            MetaFieldType::RATING => String::from("rating"),
            MetaFieldType::RICHTEXTFIELD => String::from("rich_text_field"),
            MetaFieldType::SINGLELINETEXTFIELD => String::from("single_line_text_field"),
            MetaFieldType::URL => String::from("url"),
            MetaFieldType::VOLUME => String::from("volume"),
            MetaFieldType::WEIGHT => String::from("weight"),
            MetaFieldType::COLLECTIONREFERENCE => String::from("collection_reference"),
            MetaFieldType::FILEREFERENCE => String::from("file_reference"),
            MetaFieldType::METAOBJECTREFERENCE => String::from("metaobject_reference"),
            MetaFieldType::MIXEDREFERENCE => String::from("mixed_reference"),
            MetaFieldType::PAGEREFERENCE => String::from("page_reference"),
            MetaFieldType::PRODUCTREFERENCE => String::from("product_reference"),
            MetaFieldType::VARIANTREFERENCE => String::from("variant_reference"),
            // List Types
            MetaFieldType::LISTCOLLECTIONREFERENCE => String::from("list.collection_reference"),
            MetaFieldType::LISTCOLOR => String::from("list.color"),
            MetaFieldType::LISTDATE => String::from("list.date"),
            MetaFieldType::LISTDATETIME => String::from("list.date_time"),
            MetaFieldType::LISTDATETIME => String::from("list.date_time"),
            MetaFieldType::LISTDIMENSION => String::from("list.dimension"),
            MetaFieldType::LISTFILEREFERENCE => String::from("list.file_reference"),
            MetaFieldType::LISTLINK => String::from("list.link"),
            MetaFieldType::LISTMETAOBJECTREFERENCE => String::from("list.metaobject_reference"),
            MetaFieldType::LISTMIXEDREFERENCE => String::from("list.mixed_reference"),
            MetaFieldType::LISTNUMBERINTEGER => String::from("list.number_integer"),
            MetaFieldType::LISTNUMBERDECIMAL => String::from("list.number_decimal"),
            MetaFieldType::LISTPAGEREFERENCE => String::from("list.page_reference"),
            MetaFieldType::LISTPRODUCTREFERENCE => String::from("list.product_reference"),
            MetaFieldType::LISTRATING => String::from("list.rating"),
            MetaFieldType::LISTSINGLELINETEXTFIELD => String::from("list.single_line_text_field"),
            MetaFieldType::LISTURL => String::from("list.url"),
            MetaFieldType::LISTVARIANTREFERENCE => String::from("list.variant_reference"),
            MetaFieldType::LISTVOLUME => String::from("list.volume"),
            MetaFieldType::LISTWEIGHT => String::from("list.weight"),
        }
    }
}

impl Serialize for MetaFieldType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MetaFieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetaFieldTypeVisitor;

        impl<'de> Visitor<'de> for MetaFieldTypeVisitor {
            type Value = MetaFieldType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MetaFieldType::from_str(value))
            }
        }

        deserializer.deserialize_str(MetaFieldTypeVisitor)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldCreateResponse {
    #[serde(rename="metafieldsSet")]
    pub metafields_set: Option<NewMetafields>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetafields {
    metafields: Option<Vec<NewMetafield>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMetafield {
   pub id: Option<String>,   // will be new metafield GID
   pub key: Option<String>,
   pub namespace: Option<String>,
   #[serde(rename="createdAt")]
   pub created_at: Option<DateTime<Utc>>,
   #[serde(rename="updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
   pub value: Option<MetaFieldValue>
}

// Metafield Delete Funciton
pub async fn metafield_delete_function(shop: &ShopifyCreds, metafield_gid: &str, rate_limiter: &RateLimiter)
-> Result<(), AppError>
{  
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let mutation = r#"mutation 
   metafieldDelete($input: MetafieldDeleteInput!){
      metafieldDelete(input: $input){
        deletedId
        userErrors {
            field
            message
        }
      }
   }"#;
   
   #[derive(Debug, Clone, Deserialize, Serialize)]
   struct InputVariables {
      input: Input
   }

   #[derive(Debug, Clone, Deserialize, Serialize)]
   struct Input {
      id: String,
   }
   
   let metafield_to_delete = InputVariables { input: Input { id: metafield_gid.to_string() }};
   
   match shopify_graphql_request_with_retries::<serde_json::Value, InputVariables>(gql_client.clone(), &mutation, Some(metafield_to_delete), 3, rate_limiter).await{
      Ok(response) => {
        // println!("{:?}", response);
        return Ok(())
      }, 
      Err(e) => {
        // println!("{}", e)
        Err(AppError::OtherError(e.to_string()))
      }
   }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldConnection {
    pub edges: Option<Vec<MetafieldEdge>>,
    // pub nodes: Option<Vec<Metafield>>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldEdge {
    pub node: Option<Metafield>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metafield {
    #[serde(rename="compareDigest")]
    pub compare_digest: Option<String>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub definition: Option<MetafieldDefinition>,
    pub description: Option<String>,
    pub id: Option<String>,
    #[serde(rename="jsonValue")]
    pub json_value: Option<serde_json::Value>,
    pub key: Option<String>,
    // #[serde(rename="legacyResourceId")]
    // pub legacy_resource_id: Option<String>,
    pub namespace: Option<String>,
    // pub owner: Option<HasMetafields>,
    #[serde(rename="ownerType")]
    pub owner_type: Option<MetafieldOwnerType>,
    // pub reference: Option<MetafieldReference>,
    // pub references: Option<MetafieldReferenceConnection>,
    #[serde(rename="type")]
    pub meta_type: Option<MetaFieldType>,
    #[serde(rename="updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetafieldDefinition {
    pub id: Option<String>,
}




