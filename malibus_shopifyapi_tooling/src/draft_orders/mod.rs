#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use serde::de::{self, Visitor};
use std::fmt;
use chrono::{ DateTime, Utc };
use crate::money::MoneyInput;
use crate::customer::graphQL::MailingAddressInput;
use crate::AttributeInput;
use crate::metafields::MetafieldInput;
use crate::payments::PaymentTermsInput;
use crate::money::CurrencyCode;
use crate::company::PurchasingCompanyInput;
use crate::shipping::ShippingLineInput;
use crate::products::WeightInput;

pub mod createDraftOrder;

// https://shopify.dev/docs/api/admin-graphql/2024-10/mutations/draftOrderCreate

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DraftOrderInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="acceptAutomaticDiscounts")]
    pub accept_automatic_discounts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="acceptDiscountCodesInCheckout")]
    pub accept_discount_codes_in_checkout: Option<bool>,
    #[serde(rename="appliedDiscount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_discount: Option<DraftOrderAppliedDiscountInput>,
    #[serde(rename="billingAddress")]
    pub billing_address: Option<MailingAddressInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<AttributeInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="discountCodes")]
    pub discount_codes: Option<Vec<String>>,            // DBL check could be like "..., ..., ..."
    pub email: String,                                  // Reqiured 
    #[serde(rename="lineItems")]
    pub line_items: Vec<DraftOrderLineItemInput>,       // Required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafields: Option<Vec<MetafieldInput>>,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="paymentTerms")]
    pub payment_terms: Option<PaymentTermsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,                          // E.164 format
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="poNumber")]
    pub po_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="presentmentCurrencyCode")]
    pub presentment_currency_code: Option<CurrencyCode>,
    #[serde(rename="purchasingEntity")]
    pub purchasing_entity: Option<PurchasingEntityInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="reserveInventoryUntil")]
    pub reserve_inventory_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="sessionToken")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="shippingAddress")]
    pub shipping_address: Option<MailingAddressInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="shippingLine")]
    pub shipping_line: Option<ShippingLineInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="sourceName")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,                // A comma separated list of tags that have been added to the draft order.
    #[serde(rename="taxExempt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="transformerFingerprint")]
    pub transformer_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="useCustomerDefaultAddress")]
    pub use_customer_default_address: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="visibleToCustomer")]
    pub visible_to_customer: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PurchasingEntityInput {
    #[serde(rename="customerId")]
    pub customer_id: Option<String>,                     // Shopify GID
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="purchasingCompany")]
    pub purchasing_company: Option<PurchasingCompanyInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]         // Currently Draft Orders Don't support subscriptions!
pub struct DraftOrderLineItemInput {
    #[serde(rename="appliedDiscount")]
    pub applied_discount: Option<DraftOrderAppliedDiscountInput>,
    #[serde(rename="bundleComponents")]
    pub bundle_components: Option<BundlesDraftOrderBundleLineItemComponentInput>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<AttributeInput>>,
    #[serde(rename="originalUnitPriceWithCurrency")]
    pub original_unit_price_with_currency: Option<MoneyInput>,
    pub quantity: u32,                                            // required
    #[serde(rename="requiresShipping")]
    pub requires_shipping: Option<bool>,                          // ignored when variantID provided
    pub sku: Option<String>,
    pub taxable: Option<bool>,
    pub title: Option<String>,
    pub uuid: Option<String>,
    #[serde(rename="variantId")]
    pub variant_id: Option<String>,
    pub weight: Option<WeightInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundlesDraftOrderBundleLineItemComponentInput {
    pub quantity: u32,
    pub uuid: Option<String>,
    #[serde(rename="variantId")]
    pub variant_id: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DraftOrderAppliedDiscountInput {
    #[serde(rename="amountWithCurrency")]
    pub amount_with_currency: Option<MoneyInput>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub value: Option<f64>,
    #[serde(rename="valueType")]
    pub value_type: Option<DraftOrderAppliedDiscountType>,
}

#[derive(Debug, Clone)]
pub enum DraftOrderAppliedDiscountType {
    FIXEDAMOUNT,
    PERCENTAGE,
}

impl DraftOrderAppliedDiscountType {
    pub fn from_str(discount_type: &str) ->  DraftOrderAppliedDiscountType {
        match discount_type {
            "FIXED_AMOUNT" | "fixed_amount" =>  DraftOrderAppliedDiscountType::FIXEDAMOUNT,              
            "PERCENTAGE" | "percentage" =>  DraftOrderAppliedDiscountType::PERCENTAGE,
            &_ => todo!() 
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            DraftOrderAppliedDiscountType::FIXEDAMOUNT => String::from("FIXED_AMOUNT"),             
            DraftOrderAppliedDiscountType::PERCENTAGE => String::from("PERCENTAGE"),
        }
    }
}

impl Serialize for DraftOrderAppliedDiscountType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DraftOrderAppliedDiscountType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DraftOrderAppliedDiscountTypeVisitor;

        impl<'de> Visitor<'de> for DraftOrderAppliedDiscountTypeVisitor {
            type Value = DraftOrderAppliedDiscountType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DraftOrderAppliedDiscountType::from_str(value))
            }
        }

        deserializer.deserialize_str(DraftOrderAppliedDiscountTypeVisitor)
    }
}
