#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use std::fmt::Debug;
use serde::de::{self, Visitor};
use std::fmt;
use crate::PageInfo;
use chrono::{ DateTime, Utc };
use crate::metafields::MetafieldInput;
use crate::money::MoneyV2;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanGroupResourceInput {
    #[serde(rename="productIds")]
    pub product_ids: Option<Vec<String>>,           // The IDs of the Products to add to the Selling Plan Group 
    #[serde(rename="productVariantIds")]
    pub product_variant_ids: Option<Vec<String>>,   // The IDs of the Variants to add to the Selling Plan Group
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,                      // shopify gid? 
    pub name: String,                            // eg. "Subscribe and Save" Buyer facing string which describes the selling plan content.
    pub position: Option<u32>,
    pub options: String,                    // The values of all options available on the selling plan. Selling plans are grouped together in Liquid when they're created by the same app, and have the same selling_plan_group.name and selling_plan_group.options values.
    #[serde(rename="billingPolicy")]
    pub billing_policy: SellingPlanBillingPolicyInput,
    pub category: Option<SellingPlanCategory>,
    #[serde(rename="deliveryPolicy")]
    pub delivery_policy: SellingPlanDeliveryPolicyInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,             // buyer facing string which describes the selling plan commitment.
    #[serde(rename="inventoryPolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_policy: Option<SellingPlanInventoryPolicyInput>,                
    pub metafields: Option<Vec<MetafieldInput>>,
    #[serde(rename="pricingPolicies")]
    pub pricing_policies: Vec<SellingPlanPricingPolicyInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanPricingPolicyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<SellingPlanFixedPricingPolicyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring: Option<SellingPlanRecurringPricingPolicyInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanRecurringPricingPolicyInput {
    #[serde(rename="adjustmentType")]
    pub adjustment_type: Option<SellingPlanPricingPolicyAdjustmentType>,
    #[serde(rename="adjustmentValue")]
    pub adjustment_value: Option<SellingPlanPricingPolicyValueInput>,
    #[serde(rename="afterCycle")]
    pub after_cycle: u32,               // ** REQUIRED : Cycle after which the pricing policy applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,             // Id of the pricing policy
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanFixedPricingPolicyInput {
    #[serde(rename="adjustmentType")]
    pub adjustment_type: Option<SellingPlanPricingPolicyAdjustmentType>,
    #[serde(rename="adjustmentValue")]
    pub adjustment_value: Option<SellingPlanPricingPolicyValueInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,      // Shopify GID of the pricing Policy
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanPricingPolicyValueInput {
    #[serde(rename="fixedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_value: Option<String>,      // eg. "29.99" "29.999" Scalar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum SellingPlanPricingPolicyAdjustmentType {
    PERCENTAGE,
    FIXEDAMOUNT,
    PRICE
}

impl SellingPlanPricingPolicyAdjustmentType {
    pub fn from_str(role: &str) -> SellingPlanPricingPolicyAdjustmentType {
        match role {
            "PERCENTAGE" | "Percentage" | "percentage" => SellingPlanPricingPolicyAdjustmentType::PERCENTAGE,
            "FIXED_AMOUNT" | "Fixed_Amount" | "fixed_amount" => SellingPlanPricingPolicyAdjustmentType::FIXEDAMOUNT,
            "PRICE" | "price" => SellingPlanPricingPolicyAdjustmentType::PRICE,
            _=> SellingPlanPricingPolicyAdjustmentType::PERCENTAGE
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanPricingPolicyAdjustmentType::PERCENTAGE => String::from("PERCENTAGE"),
            SellingPlanPricingPolicyAdjustmentType::FIXEDAMOUNT => String::from("FIXED_AMOUNT"),
            SellingPlanPricingPolicyAdjustmentType::PRICE => String::from("PRICE"),
        }
    }
}

impl Serialize for SellingPlanPricingPolicyAdjustmentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanPricingPolicyAdjustmentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanPricingPolicyAdjustmentTypeVisitor;

        impl<'de> Visitor<'de> for SellingPlanPricingPolicyAdjustmentTypeVisitor {
            type Value = SellingPlanPricingPolicyAdjustmentType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanPricingPolicyAdjustmentType::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanPricingPolicyAdjustmentTypeVisitor)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanInventoryPolicyInput {
    pub reserve: Option<SellingPlanReserve>
        // When to reserve inventory for the order. The value must be ON_FULFILLMENT or ON_SALE.
}

#[derive(Debug, Clone)]
pub enum SellingPlanReserve {
    ONFULFILLMENT,
    ONSALE
}

impl SellingPlanReserve {
    pub fn from_str(role: &str) -> SellingPlanReserve {
        match role {
            "ON_SALE" | "on_sale" => SellingPlanReserve::ONSALE,
            "ON_FULFILLMENT" | "on_fulfillment" | _=> SellingPlanReserve::ONFULFILLMENT,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanReserve::ONSALE => String::from("ON_SALE"),
            SellingPlanReserve::ONFULFILLMENT => String::from("ON_FULFILLMENT"),
        }
    }
}

impl Serialize for SellingPlanReserve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanReserve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanReserveVisitor;

        impl<'de> Visitor<'de> for SellingPlanReserveVisitor {
            type Value = SellingPlanReserve;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanReserve::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanReserveVisitor)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanDeliveryPolicyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<SellingPlanFixedDeliveryPolicyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring: Option<SellingPlanRecurringDeliveryPolicyInput>, 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanRecurringDeliveryPolicyInput {
    pub anchors: Option<Vec<SellingPlanAnchorInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutoff: Option<u32>,                         // a buffer period for orders to be included in cycle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<SellingPlanRecurringDeliveryPolicyIntent>,
    pub interval: Option<SellingPlanInterval>,
    #[serde(rename="intervalCount")]
    pub interval_count: Option<u32>,     // number of intervals between deliveries
    #[serde(rename="preAnchorBehavior")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_anchor_behavior: Option<SellingPlanRecurringDeliveryPolicyPreAnchorBehavior>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SellingPlanRecurringDeliveryPolicyPreAnchorBehavior {
    ASAP,
    NEXT,
}

impl SellingPlanRecurringDeliveryPolicyPreAnchorBehavior {
    pub fn from_str(role: &str) -> SellingPlanRecurringDeliveryPolicyPreAnchorBehavior {
        match role {
            "NEXT" | "next" => SellingPlanRecurringDeliveryPolicyPreAnchorBehavior::NEXT,
            "ASAP" | "asap" | _=> SellingPlanRecurringDeliveryPolicyPreAnchorBehavior::ASAP,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanRecurringDeliveryPolicyPreAnchorBehavior::ASAP => String::from("ASAP"),
            SellingPlanRecurringDeliveryPolicyPreAnchorBehavior::NEXT => String::from("NEXT"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SellingPlanRecurringDeliveryPolicyIntent {
    FULFILLMENTBEGIN
}

impl SellingPlanRecurringDeliveryPolicyIntent {
    pub fn from_str(role: &str) -> SellingPlanRecurringDeliveryPolicyIntent {
        match role {
            "FULFILLMENT_BEGIN" | "fulfillment_begin" | _=> SellingPlanRecurringDeliveryPolicyIntent::FULFILLMENTBEGIN,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanRecurringDeliveryPolicyIntent::FULFILLMENTBEGIN => String::from("FULFILLMENT_BEGIN"),
        }
    }
}

impl Serialize for SellingPlanRecurringDeliveryPolicyIntent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanRecurringDeliveryPolicyIntent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanRecurringDeliveryPolicyIntentVisitor;

        impl<'de> Visitor<'de> for SellingPlanRecurringDeliveryPolicyIntentVisitor {
            type Value = SellingPlanRecurringDeliveryPolicyIntent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanRecurringDeliveryPolicyIntent::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanRecurringDeliveryPolicyIntentVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanFixedDeliveryPolicyInput {
    pub anchors: Option<Vec<SellingPlanAnchorInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutoff: Option<u32>,                        // a buffer period for orders to be included in cycle
    #[serde(rename="fulfillmentExactTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_exact_time: Option<DateTime<Utc>>,
    #[serde(rename="fulfillmentTrigger")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_trigger: Option<SellingPlanFulfillmentTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<SellingPlanFixedDeliveryPolicyIntent>,
    #[serde(rename="preAnchorBehavior")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_anchor_behavior: Option<SellingPlanFixedDeliveryPolicyPreAnchorBehavior>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SellingPlanFixedDeliveryPolicyPreAnchorBehavior {
    ASAP,
    NEXT,
}

impl SellingPlanFixedDeliveryPolicyPreAnchorBehavior {
    pub fn from_str(role: &str) -> SellingPlanFixedDeliveryPolicyPreAnchorBehavior {
        match role {
            "NEXT" | "next" => SellingPlanFixedDeliveryPolicyPreAnchorBehavior::NEXT,
            "ASAP" | "asap" | _=> SellingPlanFixedDeliveryPolicyPreAnchorBehavior::ASAP,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanFixedDeliveryPolicyPreAnchorBehavior::ASAP => String::from("ASAP"),
            SellingPlanFixedDeliveryPolicyPreAnchorBehavior::NEXT => String::from("NEXT"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SellingPlanFixedDeliveryPolicyIntent {
    FULFILLMENTBEGIN
}

impl SellingPlanFixedDeliveryPolicyIntent {
    pub fn from_str(role: &str) -> SellingPlanFixedDeliveryPolicyIntent {
        match role {
            "FULFILLMENT_BEGIN" | "fulfillment_begin" | _=> SellingPlanFixedDeliveryPolicyIntent::FULFILLMENTBEGIN,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanFixedDeliveryPolicyIntent::FULFILLMENTBEGIN => String::from("FULFILLMENT_BEGIN"),
        }
    }
}

impl Serialize for SellingPlanFixedDeliveryPolicyIntent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanFixedDeliveryPolicyIntent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanFixedDeliveryPolicyIntentVisitor;

        impl<'de> Visitor<'de> for SellingPlanFixedDeliveryPolicyIntentVisitor {
            type Value = SellingPlanFixedDeliveryPolicyIntent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanFixedDeliveryPolicyIntent::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanFixedDeliveryPolicyIntentVisitor)
    }
}


#[derive(Debug, Clone)]
pub enum SellingPlanFulfillmentTrigger {
    ANCHOR,
    ASAP,
    EXACTTIME,
    UNKNOWN
}

impl SellingPlanFulfillmentTrigger {
    pub fn from_str(role: &str) -> SellingPlanFulfillmentTrigger {
        match role {
            "ANCHOR" | "anchor" => SellingPlanFulfillmentTrigger::ANCHOR,
            "ASAP" | "asap" => SellingPlanFulfillmentTrigger::ASAP,
            "EXACT_TIME" | "exact_time" => SellingPlanFulfillmentTrigger::EXACTTIME,
            "UNKNOWN" | "unknown" | _=> SellingPlanFulfillmentTrigger::UNKNOWN,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanFulfillmentTrigger::ANCHOR => String::from("ANCHOR"),
            SellingPlanFulfillmentTrigger::ASAP => String::from("ASAP"),
            SellingPlanFulfillmentTrigger::EXACTTIME => String::from("EXACT_TIME"),
            SellingPlanFulfillmentTrigger::UNKNOWN => String::from("UNKNOWN")
        }
    }
}

impl Serialize for SellingPlanFulfillmentTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanFulfillmentTrigger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanFulfillmentTriggerVisitor;

        impl<'de> Visitor<'de> for SellingPlanFulfillmentTriggerVisitor {
            type Value = SellingPlanFulfillmentTrigger;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanFulfillmentTrigger::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanFulfillmentTriggerVisitor)
    }
}



#[derive(Debug, Clone)]
pub enum SellingPlanCategory {
    OTHER,
    PREORDER,
    SUBSCRIPTION,
    TRYBEFOREYOUBUY
}

impl SellingPlanCategory {
    pub fn from_str(role: &str) -> SellingPlanCategory {
        match role {
            "OTHER" | "other" => SellingPlanCategory::OTHER,
            "PRE_ORDER" | "pre_order" => SellingPlanCategory::PREORDER,
            "TRY_BEFORE_YOU_BUY" | "try_before_you_buy" => SellingPlanCategory::TRYBEFOREYOUBUY,
            "SUBSCRIPTION" | "subscription" | _=>SellingPlanCategory::SUBSCRIPTION,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanCategory::OTHER => String::from("OTHER"),
            SellingPlanCategory::PREORDER => String::from("PRE_ORDER"),
            SellingPlanCategory::SUBSCRIPTION => String::from("SUBSCRIPTION"),
            SellingPlanCategory::TRYBEFOREYOUBUY => String::from("TRY_BEFORE_YOU_BUY")
        }
    }
}

impl Serialize for SellingPlanCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanCategoryVisitor;

        impl<'de> Visitor<'de> for SellingPlanCategoryVisitor {
            type Value = SellingPlanCategory;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanCategory::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanCategoryVisitor)
    }
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanBillingPolicyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<SellingPlanFixedBillingPolicyInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring: Option<SellingPlanRecurringBillingPolicyInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanRecurringBillingPolicyInput {
   pub anchors: Option<Vec<SellingPlanAnchorInput>>,
   pub interval: Option<SellingPlanInterval>, 
   #[serde(rename="intervalCount")]
   pub intervalCount: Option<u32>,              // number of intervals between billings
   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(rename="maxCycles")]
   pub max_cycles: Option<u32>,                 // maximum number of billing iterations
   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(rename="minCycles")]
   pub min_cycles: Option<u32>,                 // minimum number of billing iterations 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SellingPlanInterval {
    DAY,
    WEEK,
    MONTH,
    YEAR,
}

impl SellingPlanInterval {
    pub fn from_str(role: &str) -> SellingPlanInterval {
        match role {
            "DAY" | "Day" | "day" => SellingPlanInterval::DAY,
            "WEEK"| "Week" | "week" => SellingPlanInterval::WEEK,
            "MONTH" |"Month" | "month" => SellingPlanInterval::MONTH,
            "YEAR" | "Year" | "year" => SellingPlanInterval::YEAR,
            _ => SellingPlanInterval::WEEK
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanInterval::DAY => String::from("DAY"),
            SellingPlanInterval::WEEK => String::from("WEEK"),
            SellingPlanInterval::MONTH => String::from("MONTH"),
            SellingPlanInterval::YEAR => String::from("YEAR"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanAnchorInput {
    #[serde(rename="cutoffDay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cut_off_day: Option<u32>,
        // If type is WEEKDAY, then the value must be between 1-7. Shopify interprets the days of the week according to ISO 8601, where 1 is Monday.                  
        // If type is MONTHDAY, then the value must be between 1-31.
        // If type is YEARDAY, then the value must be null.
        // This field should only be set if the cutoff field for the delivery policy is null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u32>,
        // The day of the anchor.
        // If type is WEEKDAY, then the value must be between 1-7. Shopify interprets the days of the week according to ISO 8601, where 1 is Monday.
        // If type isn't WEEKDAY, then the value must be between 1-31.   
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<u32>,
        // The month of the anchor. If type is different than YEARDAY, then the value must be null or between 1-12.
    #[serde(rename="type")]
    pub selling_plan_anchor_type: SellingPlanAnchorType
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SellingPlanAnchorType {
    MONTHDAY,  //  Which day of the month, between 1-31.
    WEEKDAY,   //  Which day of the week, between 1-7.
    YEARDAY,   //  Which days of the month and year, month between 1-12, and day between 1-31.
}

impl SellingPlanAnchorType {
    pub fn from_str(role: &str) -> SellingPlanAnchorType {
        match role {
            "YEARDAY" | "yearday" => SellingPlanAnchorType::YEARDAY,
            "WEEKDAY" | "weekday" => SellingPlanAnchorType::WEEKDAY,
            "MONTHDAY" | "monthday" | _=> SellingPlanAnchorType::MONTHDAY,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanAnchorType::YEARDAY => String::from("YEARDAY"),
            SellingPlanAnchorType::WEEKDAY => String::from("WEEKDAY"),
            SellingPlanAnchorType::MONTHDAY => String::from("MONTHDAY")
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanFixedBillingPolicyInput {
    #[serde(rename="checkoutCharge")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_charge: Option<SellingPlanCheckoutChargeInput>,
    #[serde(rename="remainingBalanceChargeExactTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_balance_charge_exact_time: Option<DateTime<Utc>>,
    #[serde(rename="remainingBalanceChargeTimeAfterCheckout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_balance_charge_time_after_checkout: Option<String>,  // The period after capturing the payment for the amount due (remainingBalanceChargeTrigger), and before capturing the full payment. Expressed as an ISO8601 duration.   
    #[serde(rename="remainingBalanceChargeTrigger")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_balance_charge_trigger: Option<SellingPlanRemainingBalanceChargeTrigger>,
}

#[derive(Debug, Clone)]
pub enum SellingPlanRemainingBalanceChargeTrigger {
    EXACTTIME,
    NOREMAININGBALANCE,
    TIMEAFTERCHECKOUT,
}

impl SellingPlanRemainingBalanceChargeTrigger {
    pub fn from_str(role: &str) -> SellingPlanRemainingBalanceChargeTrigger {
        match role {
            "NO_REMAINING_BALANCE" | "no_remaining_balance" => SellingPlanRemainingBalanceChargeTrigger::NOREMAININGBALANCE,
            "TIME_AFTER_CHECKOUT" | "time_after_checkout" => SellingPlanRemainingBalanceChargeTrigger::TIMEAFTERCHECKOUT,
            "EXACT_TIME" | "exact_time" | _=> SellingPlanRemainingBalanceChargeTrigger::EXACTTIME,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SellingPlanRemainingBalanceChargeTrigger::EXACTTIME => String::from("EXACT_TIME"),
            SellingPlanRemainingBalanceChargeTrigger::NOREMAININGBALANCE => String::from("NO_REMAINING_BALANCE"),
            SellingPlanRemainingBalanceChargeTrigger::TIMEAFTERCHECKOUT => String::from("TIME_AFTER_CHECKOUT")
        }
    }
}

impl Serialize for SellingPlanRemainingBalanceChargeTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SellingPlanRemainingBalanceChargeTrigger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SellingPlanRemainingBalanceChargeTriggerVisitor;

        impl<'de> Visitor<'de> for SellingPlanRemainingBalanceChargeTriggerVisitor {
            type Value = SellingPlanRemainingBalanceChargeTrigger;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid meta field type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SellingPlanRemainingBalanceChargeTrigger::from_str(value))
            }
        }

        deserializer.deserialize_str(SellingPlanRemainingBalanceChargeTriggerVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanCheckoutChargeInput {
    #[serde(rename="type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_charge_type: Option<SellingPlanCheckoutChargeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<SellingPlanCheckoutChargeValueInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanCheckoutChargeValueInput {
    #[serde(rename="fixedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_value: Option<String>,     // eg. "29.99", "29.999" Scalar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SellingPlanCheckoutChargeType {
    PERCENTAGE,
    PRICE,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanAnchor {
    #[serde(rename="cutoffDay")]
    pub cut_off_day: Option<u32>,
    pub day: Option<u32>,
    pub month: Option<u32>,
    #[serde(rename="type")]
    pub selling_plan_anchor_type: Option<SellingPlanAnchorType>
}

// ** BASE Struct **:
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanGroupConnection {
    pub edges: Option<Vec<SellingPlanGroupEdge>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanGroupEdge {
    pub node: Option<SellingPlanGroup>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanGroup {
    pub id: Option<String>,
    #[serde(rename="sellingPlans")]
    pub selling_plans: Option<SellingPlanConnection>,
    // Add any other fields you need from the Shopify API for SellingPlanGroup
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanConnection {
    pub edges: Option<Vec<SellingPlanEdge>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanEdge {
    pub node: Option<SellingPlan>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlan {
    pub id: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub options: Option<String>,
    pub position: Option<u32>,
    pub category: Option<SellingPlanCategory>,
    #[serde(rename="billingPolicy")]
    pub billing_policy: Option<SellingPlanBillingPolicyInput>,
    #[serde(rename="deliveryPolicy")]
    pub delivery_policy: Option<SellingPlanDeliveryPolicyInput>,
    #[serde(rename="inventoryPolicy")]
    pub inventory_policy: Option<SellingPlanInventoryPolicyInput>,
    #[serde(rename="pricingPolicies")]
    pub pricing_policies: Option<Vec<SellingPlanPricingPolicyInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanPricingPolicyPercentageValue {
    #[serde(rename = "afterCycle")]
    pub after_cycle: i32,
    #[serde(rename = "computedPrice")]
    pub computed_price: MoneyV2,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
pub enum SellingPlanPricingPolicyAdjustmentValue {
    MoneyV2(MoneyV2),
    SellingPlanPricingPolicyPercentageValue(SellingPlanPricingPolicyPercentageValue),
}


/*
use crate::products::Product;
use crate::variants::ProductVariant;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanGroupConnection {
    // #[serde(rename="DeliveryProfile.sellingPlanGroups")]
    // pub delivery_profile_selling_plan_groups: Option<DeliveryProfile>,
    #[serde(rename="Product.sellingPlanGroups")]
    pub product_selling_plan_groups: Option<Product>,
    #[serde(rename="ProductVariant.sellingPlanGroups")]
    pub product_variant_selling_plan_groups: Option<ProductVariant>,
}*/


