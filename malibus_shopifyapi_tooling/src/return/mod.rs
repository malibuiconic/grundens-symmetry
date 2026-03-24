#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use crate::PageInfo;
use crate::orders::LineItem;
use crate::orders::Order;
use crate::refund::RefundConnection;
use crate::money::MoneyBag;
use crate::refund::RefundDutyInput;

// https://shopify.dev/docs/api/admin-graphql/latest/objects/Return
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Return {
    pub decline: Option<ReturnDecline>,
    #[serde(rename = "exchangeLineItems")]
    pub exchange_line_items: Option<ExchangeLineItemConnection>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub order: Option<Order>,
    pub refunds: Option<RefundConnection>,
    #[serde(rename = "returnLineItems")]
    pub return_line_items: Option<ReturnLineItemTypeConnection>,
    #[serde(rename = "returnShippingFees")]
    pub return_shipping_fees: Option<Vec<ReturnShippingFee>>,
    // #[serde(rename = "reverseFulfillmentOrderConnection")]
    // pub reverse_fulfillment_order_connection: Option<ReverseFulfillmentOrderConnection>,
    pub status: Option<ReturnStatus>,
    #[serde(rename = "suggestedRefund")]
    pub suggested_refund: Option<SuggestedReturnRefund>,
    #[serde(rename = "totalQuantity")]
    pub total_quantity: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SuggestedReturnRefund {
    #[serde(rename = "returnRefundLineItems")]
    pub return_refund_line_items: Option<Vec<ReturnRefundLineItemInput>>,
    // #[serde(rename = "refundShipping")]
    // pub refund_shipping: Option<RefundShippingInput>,
    #[serde(rename = "refundDuties")]
    pub refund_duties: Option<Vec<RefundDutyInput>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnRefundLineItemInput {
    pub quantity: Option<u32>,
    #[serde(rename = "returnLineItemId")]
    pub return_line_item_id: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReturnStatus {
    CANCELED,
    CLOSED,
    DECLINED,
    OPEN,
    REQUESTED,
}

impl ReturnStatus {
    pub fn from_str(status: &str) -> ReturnStatus {
        match status.to_uppercase().as_str() {
            "CANCELED" => ReturnStatus::CANCELED,
            "CLOSED" => ReturnStatus::CLOSED,
            "DECLINED" => ReturnStatus::DECLINED,
            "OPEN" => ReturnStatus::OPEN,
            "REQUESTED"| _=> ReturnStatus::REQUESTED,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ReturnStatus::CANCELED => String::from("CANCELED"),
            ReturnStatus::CLOSED => String::from("CLOSED"),
            ReturnStatus::DECLINED => String::from("DECLINED"),
            ReturnStatus::OPEN => String::from("OPEN"),
            ReturnStatus::REQUESTED => String::from("REQUESTED"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnShippingFee {
    #[serde(rename = "amountSet")]
    pub amount_set: Option<MoneyBag>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnLineItemTypeConnection {
    pub edges: Option<Vec<ReturnLineItemTypeEdge>>,     
    // pub nodes: Option<Vec<ReturnLineItemType>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnLineItemTypeEdge {
    pub node: Option<ReturnLineItemType>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnLineItemType {
    #[serde(rename = "customerNote")]
    pub customer_note: Option<String>,
    pub id: Option<String>,
    pub quantity: Option<u32>,
    #[serde(rename = "refundableQuantity")]
    pub refundable_quantity: Option<u32>,
    #[serde(rename = "refundedQuantity")]
    pub refunded_quantity: Option<u32>,
    #[serde(rename = "returnReason")]
    pub return_reason: Option<ReturnReason>,
    #[serde(rename = "returnReasonNote")]
    pub return_reason_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReturnReason {
    COLOR,
    DEFECTIVE,
    NOTASDESCRIBED,
    OTHER,
    SIZETOOLARGE,
    SIZETOOSMALL,
    STYLE,
    UNKNOWN,
    UNWANTED,
    WRONGITEM,
}

impl ReturnReason {
    pub fn from_str(status: &str) -> ReturnReason {
        match status.to_uppercase().as_str() {
            "COLOR" => ReturnReason::COLOR,
            "DEFECTIVE" => ReturnReason::DEFECTIVE,
            "NOT_AS_DESCRIBED" => ReturnReason::NOTASDESCRIBED,
            "OTHER" => ReturnReason::OTHER,
            "SIZE_TOO_LARGE" => ReturnReason::SIZETOOLARGE,
            "SIZE_TOO_SMALL" => ReturnReason::SIZETOOSMALL,
            "STYLE" => ReturnReason::STYLE,
            "UNWANTED" => ReturnReason::UNWANTED,
            "WRONG_ITEM" => ReturnReason::WRONGITEM,
            "UNKNOWN"| _=> ReturnReason::UNKNOWN,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ReturnReason::COLOR => String::from("COLOR"),
            ReturnReason::DEFECTIVE => String::from("DEFECTIVE"),
            ReturnReason::NOTASDESCRIBED => String::from("NOT_AS_DESCRIBED"),
            ReturnReason::OTHER => String::from("OTHER"),
            ReturnReason::SIZETOOLARGE => String::from("SIZE_TOO_LARGE"),
            ReturnReason::SIZETOOSMALL => String::from("SIZE_TOO_SMALL"),
            ReturnReason::STYLE => String::from("STYLE"),
            ReturnReason::UNWANTED => String::from("UNWANTED"),
            ReturnReason::UNKNOWN => String::from("UNKNOWN"),
            ReturnReason::WRONGITEM => String::from("WRONG_ITEM"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeLineItemConnection {
    pub edges: Option<Vec<ExchangeLineItemEdge>>,     // includeRemovedItems (bool) argument default false
    // pub nodes: Option<Vec<ExchangeLineItemEdge>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeLineItemEdge {
    pub node: Option<ExchangeLineItem>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeLineItem {
    pub id: Option<String>,
    #[serde(rename = "lineItem")]
    pub line_item: Option<LineItem>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnDecline {
    pub note: Option<String>,
    pub reason: Option<ReturnDeclineReason>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReturnDeclineReason {
    FINALSALE,
    OTHER,
    RETURNPERIODENDED
}

impl ReturnDeclineReason {
    pub fn from_str(status: &str) -> ReturnDeclineReason {
        match status.to_uppercase().as_str() {
            "FINAL_SALE" => ReturnDeclineReason::FINALSALE,
            "RETURN_PERIOD_ENDED" => ReturnDeclineReason::RETURNPERIODENDED,
            "OTHER"| _=> ReturnDeclineReason::OTHER,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ReturnDeclineReason::FINALSALE => String::from("FINAL_SALE"),
            ReturnDeclineReason::RETURNPERIODENDED => String::from("RETURN_PERIOD_ENDED"),
            ReturnDeclineReason::OTHER => String::from("OTHER"),
        }
    }
}