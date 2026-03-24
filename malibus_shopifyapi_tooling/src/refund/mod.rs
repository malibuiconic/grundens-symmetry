#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use crate::money::CurrencyCode;
use crate::money::Money;
use crate::money::MoneyBag;
use chrono::{ DateTime, Utc };
use crate::orders::OrderTransactionKind;
use crate::customer::graphQL::CountryCode;
use crate::taxes::TaxLine;
use crate::orders::Order;
use crate::PageInfo;
use crate::orders::LineItem;
use crate::user::StaffMember;
use crate::orders::OrderTransactionConnection;
use crate::location::Location;
use crate::r#return::Return;

pub mod refundCreate;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RefundInput {
    pub currency: Option<CurrencyCode>,
    #[serde(rename = "discrepancyReason")]
    pub discrepancy_reason: Option<OrderAdjustmentInputDiscrepancyReason>,
    pub note: Option<String>,
    pub notify: Option<bool>,
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    #[serde(rename = "refundDuties")]
    pub refund_duties: Option<Vec<RefundDutyInput>>,
    #[serde(rename = "refundLineItems")]
    pub refund_line_items: Option<Vec<RefundLineItemInput>>,
    pub shipping: Option<ShippingRefundInput>,
    pub transactions: Option<Vec<OrderTransactionInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderTransactionInput {
    pub amount: Option<Money>,
    pub gateway: Option<String>,
    pub kind: Option<OrderTransactionKind>,
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShippingRefundInput {
    pub amount: Option<Money>,
    #[serde(rename = "fullRefund")]
    pub full_refund: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundLineItemInput {
    #[serde(rename = "lineItemId")]
    pub line_item_id: Option<String>,
    #[serde(rename = "locationId")]
    pub location_id: Option<String>,
    pub quantity: Option<u32>,
    #[serde(rename = "restockType")]
    pub restock_type: Option<RefundLineItemRestockType>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RefundLineItemRestockType {
    CANCEL,
    LEGACYRESTOCK,
    NORESTOCK,
    RETURN,
}

impl RefundLineItemRestockType {
    pub fn from_str(status: &str) -> RefundLineItemRestockType {
        match status.to_uppercase().as_str() {
            "CANCEL" => RefundLineItemRestockType::CANCEL,
            "LEGACY_RESTOCK" => RefundLineItemRestockType::LEGACYRESTOCK,
            "NO_RESTOCK" => RefundLineItemRestockType::NORESTOCK,
            "RETURN"| _=> RefundLineItemRestockType::RETURN,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RefundLineItemRestockType::CANCEL => String::from("CANCEL"),
            RefundLineItemRestockType::LEGACYRESTOCK => String::from("LEGACY_RESTOCK"),
            RefundLineItemRestockType::NORESTOCK => String::from("NO_RESTOCK"),
            RefundLineItemRestockType::RETURN => String::from("RETURN"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundDutyInput {
    #[serde(rename = "dutyId")]
    pub duty_id: Option<String>,
    #[serde(rename = "refundType")]
    pub refund_type: Option<RefundDutyRefundType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RefundDutyRefundType {
    FULL,
    PROPORTIONAL,
}

impl RefundDutyRefundType {
    pub fn from_str(status: &str) -> RefundDutyRefundType {
        match status.to_uppercase().as_str() {
            "PROPORTIONAL" => RefundDutyRefundType::PROPORTIONAL,
            "FULL"| _=> RefundDutyRefundType::FULL,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RefundDutyRefundType::PROPORTIONAL => String::from("PROPORTIONAL"),
            RefundDutyRefundType::FULL => String::from("FULL"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OrderAdjustmentInputDiscrepancyReason{
    CUSTOMER,
    DAMAGE,
    OTHER,
    RESTOCK,
}

impl OrderAdjustmentInputDiscrepancyReason {
    pub fn from_str(status: &str) -> OrderAdjustmentInputDiscrepancyReason {
        match status.to_uppercase().as_str() {
            "CUSTOMER" => OrderAdjustmentInputDiscrepancyReason::CUSTOMER,
            "DAMAGE" => OrderAdjustmentInputDiscrepancyReason::DAMAGE,
            "RESTOCK" => OrderAdjustmentInputDiscrepancyReason::RESTOCK,
            "OTHER"| _=> OrderAdjustmentInputDiscrepancyReason::OTHER,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderAdjustmentInputDiscrepancyReason::CUSTOMER => String::from("CUSTOMER"),
            OrderAdjustmentInputDiscrepancyReason::DAMAGE => String::from("DAMAGE"),
            OrderAdjustmentInputDiscrepancyReason::RESTOCK => String::from("RESTOCK"),
            OrderAdjustmentInputDiscrepancyReason::OTHER => String::from("OTHER"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Refund {
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub duties: Option<RefundDuty>,
    pub id: Option<String>,
    #[serde(rename = "legacyResourceId")]
    pub legacy_resource_id: Option<u64>,
    pub note: Option<String>,
    pub order: Option<Order>,
    #[serde(rename = "orderAdjustments")]
    pub order_adjustments: Option<OrderAdjustmentConnection>,
    #[serde(rename = "refundLineItems")]
    pub refund_line_items: Option<RefundLineItemConnection>,
    // #[serde(rename = "refundShippingLines")]
    // pub refund_shipping_lines: Option<RefundShippingLineConnection>,
    #[serde(rename = "return")]
    pub refund_return: Option<Return>,
    #[serde(rename = "staffMember")]
    pub staff_member: Option<StaffMember>,
    #[serde(rename = "totalRefundedSet")]
    pub total_refunded_set: Option<MoneyBag>,
    pub transactions: Option<OrderTransactionConnection>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    // #[serde(rename = "totalRefunded")]
    // pub total_refunded: Option<MoneyV2>,  // deprecated
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundLineItemConnection {
    pub edges: Option<Vec<RefundLineItemEdge>>,
    // pub nodes: Option<Vec<RefundLineItem>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundLineItemEdge {
    pub node: Option<RefundLineItem>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundLineItem {
    pub id: Option<String>,
    #[serde(rename = "lineItem")]
    pub line_item: Option<LineItem>,
    pub location: Option<Location>,
    #[serde(rename = "priceSet")]
    pub price_set: Option<MoneyBag>,
    pub quantity: Option<u32>,
    pub restocked: Option<bool>,
    #[serde(rename = "restockType")]
    pub restock_type: Option<RefundLineItemRestockType>,
    #[serde(rename = "subtotalSet")]
    pub subtotal_set: Option<MoneyBag>,
    #[serde(rename = "totalTaxSet")]
    pub total_tax_set: Option<MoneyBag>,
    // pub price: Option<Money>, // deprecated
    // pub subtotal: Option<Money>, //deprecated
    // pub totalTax: Option<Money>, // deprecated
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderAdjustmentConnection {
    pub edges: Option<Vec<OrderAdjustmentEdge>>,
    // pub nodes: Option<Vec<OrderAdjustment>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderAdjustmentEdge {
    pub node: Option<OrderAdjustment>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderAdjustment{
    #[serde(rename = "amountSet")]
    pub amount_set: Option<MoneyBag>,
    pub id: Option<String>,
    pub reason: Option<OrderAdjustmentDiscrepancyReason>,
    #[serde(rename = "taxAmountSet")]
    pub tax_amount_set: Option<MoneyBag>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OrderAdjustmentDiscrepancyReason {
    CUSTOMER,
    DAMAGE,
    FULLRETURNBALANCINGADJUSTMENT,
    PENDINGREFUNDDISCREPANCY,
    REFUNDDISCREPANCY,
    RESTOCK,
}

impl OrderAdjustmentDiscrepancyReason {
    pub fn from_str(status: &str) -> OrderAdjustmentDiscrepancyReason {
        match status.to_uppercase().as_str() {
            "CUSTOMER" => OrderAdjustmentDiscrepancyReason::CUSTOMER,
            "DAMAGE" => OrderAdjustmentDiscrepancyReason::DAMAGE,
            "FULL_RETURN_BALANCING_ADJUSTMENT" => OrderAdjustmentDiscrepancyReason::FULLRETURNBALANCINGADJUSTMENT,
            "PENDING_REFUND_ADJUSTMENT" => OrderAdjustmentDiscrepancyReason::PENDINGREFUNDDISCREPANCY,
            "REFUND_DISCREPANCY" => OrderAdjustmentDiscrepancyReason::REFUNDDISCREPANCY,
            "RESTOCK" | _=> OrderAdjustmentDiscrepancyReason::RESTOCK,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderAdjustmentDiscrepancyReason::CUSTOMER => String::from("CUSTOMER"),
            OrderAdjustmentDiscrepancyReason::DAMAGE => String::from("DAMAGE"),
            OrderAdjustmentDiscrepancyReason::FULLRETURNBALANCINGADJUSTMENT => String::from("FULL_RETURN_BALANCING_ADJUSTMENT"),
            OrderAdjustmentDiscrepancyReason::PENDINGREFUNDDISCREPANCY => String::from("PENDING_REFUND_ADJUSTMENT"),
            OrderAdjustmentDiscrepancyReason::REFUNDDISCREPANCY => String::from("REFUND_DISCREPANCY"),
            OrderAdjustmentDiscrepancyReason::RESTOCK => String::from("RESTOCK"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundDuty {
    #[serde(rename = "amountSet")]
    pub amount_set: Option<MoneyBag>,
    #[serde(rename = "originalDuty")]
    pub original_duty: Option<Duty>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Duty {
    #[serde(rename = "countryCodeOfOrigin")]
    pub country_code_of_origin: Option<CountryCode>,
    #[serde(rename = "harmonizedSystemCode")]
    pub harmonized_system_code: Option<String>,
    pub id: Option<String>,
    pub price: Option<MoneyBag>,
    #[serde(rename = "taxLines")]
    pub tax_lines: Option<Vec<TaxLine>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundConnection {
    pub edges: Option<Vec<RefundEdge>>,
    // pub nodes: Option<Vec<RefundLineItem>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefundEdge {
    pub node: Option<Refund>,
    pub cursor: Option<String>,
}