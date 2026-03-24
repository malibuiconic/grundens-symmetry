#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use std::fmt::Debug;
use serde::de::{self, Visitor};
use std::fmt;
use crate::money::Decimal;
use chrono::{ DateTime, Utc };
use crate::selling_plans::SellingPlanInterval;
use crate::selling_plans::SellingPlanAnchor;
use crate::money::CurrencyCode;
use crate::payments::CustomerPaymentMethod;
use crate::PageInfo;
use crate::orders::Order;
use crate::AttributeInput;
use crate::app::App;
use crate::customer::Customer;
use crate::money::MoneyV2;
use crate::image::Image;
use crate::count::Count;
use crate::discount::DiscountTargetType;
use crate::discount::DiscountType;
use crate::orders::OrderConnection;
use crate::orders::OrderTransactionConnection;
use crate::selling_plans::SellingPlanPricingPolicyAdjustmentType;
use crate::selling_plans::SellingPlanPricingPolicyAdjustmentValue;
use crate::selling_plans::SellingPlanAnchorInput;
use crate::customer::graphQL::MailingAddressInput;

// https://shopify.dev/docs/api/admin-graphql/2025-01/objects/SubscriptionDraft

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDraft {
    #[serde(rename="billingCycle")]
    pub billing_cycle: Option<SubscriptionBillingCycle>,
    #[serde(rename="billingPolicy")]
    pub billing_policy: Option<SubscriptionBillingPolicy>,
    #[serde(rename="concatenatedBillingCycles")]
    pub concatenated_billing_cycles: Option<SubscriptionBillingCycleConnection>,
    #[serde(rename="currencyCode")]
    pub currency_code: Option<CurrencyCode>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<Attribute>>,
    pub customer: Option<Customer>,
    #[serde(rename="customerPaymentMethod")]
    pub customer_payment_method: Option<CustomerPaymentMethod>,
    #[serde(rename="deliveryMethod")]
    pub delivery_method: Option<SubscriptionDeliveryMethod>,
    #[serde(rename="deliveryOptions")]
    pub delivery_options: Option<SubscriptionDeliveryOptionResult>,
    #[serde(rename="deliveryPolicy")]
    pub delivery_policy: Option<SubscriptionDeliveryPolicy>,
    #[serde(rename="deliveryPrice")]
    pub delivery_price: Option<MoneyV2>,
    pub discounts: Option<SubscriptionDiscountConnection>,
    #[serde(rename="discountsAdded")]
    pub discounts_added: Option<SubscriptionDiscountConnection>,
    #[serde(rename="discountsRemoved")]
    pub discounts_removed: Option<SubscriptionDiscountConnection>,
    #[serde(rename="discountsUpdated")]
    pub discounts_updated: Option<SubscriptionDiscountConnection>,
    pub id: Option<String>,
    pub lines: Option<SubscriptionLineConnection>,
    #[serde(rename="linesAdded")]
    pub lines_added: Option<SubscriptionLineConnection>,
    #[serde(rename="linesRemoved")]
    pub lines_removed: Option<SubscriptionLineConnection>,
    #[serde(rename="nextBillingDate")]
    pub next_billing_date: Option<DateTime<Utc>>,
    pub note: Option<String>,
    #[serde(rename="originalContract")]
    pub original_contract: Option<SubscriptionContract>,
    pub status: Option<SubscriptionContractSubscriptionStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountConnection {
    pub edges: Option<Vec<SubscriptionDiscountEdge>>,
    // pub nodes: Option<Vec<SubscriptionDiscount>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountEdge {
    pub node: Option<SubscriptionDiscount>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
pub enum SubscriptionDeliveryOptionResult {
    SubscriptionDeliveryOptionResultFailure(SubscriptionDeliveryOptionResultFailure),
    SubscriptionDeliveryOptionResultSuccess(SubscriptionDeliveryOptionResultSuccess),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
pub enum SubscriptionDeliveryOption {
    SubscriptionLocalDeliveryOption(SubscriptionLocalDeliveryOption),
    SubscriptionPickupOption(SubscriptionPickupOption),
    SubscriptionShippingOption(SubscriptionShippingOption),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryOptionResultFailure {
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryOptionResultSuccess {
    #[serde(rename = "deliveryOptions")]
    pub delivery_options: Vec<SubscriptionDeliveryOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionLocalDeliveryOption {
    // Add fields as needed for local delivery
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionPickupOption {
    // Add fields as needed for pickup
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionShippingOption {
    pub code: String,
    pub description: Option<String>,
    #[serde(rename = "phoneRequired")]
    pub phone_required: Option<bool>,
    #[serde(rename = "presentmentTitle")]
    pub presentment_title: Option<String>,
    pub price: Option<MoneyV2>,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attribute {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingPolicy {
    pub anchors: Option<Vec<SellingPlanAnchor>>,
    pub interval: Option<SellingPlanInterval>,
    #[serde(rename="intervalCount")]
    pub interval_count: Option<u32>,
    #[serde(rename="maxCycles")]
    pub max_cycles: Option<u32>,
    #[serde(rename="minCycles")]
    pub min_cycles: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingCycle {
    #[serde(rename="billingAttemptExpectedDate")]
    pub billing_attempt_expected_date: Option<DateTime<Utc>>,
    #[serde(rename="billingAttempts")]
    pub billing_attempts: Option<SubscriptionBillingAttemptsConnection>,
    #[serde(rename="cycleEndAt")]
    pub cycle_end_at: Option<DateTime<Utc>>,
    #[serde(rename="cycleIndex")]
    pub cycle_index: Option<u32>,
    #[serde(rename="cycleStartAt")]
    pub cycle_start_at: Option<DateTime<Utc>>,
    pub edited: Option<bool>,
    #[serde(rename="editedContract")]
    pub edited_contract: Option<SubscriptionBillingCycleEditedContract>,
    pub skipped: Option<bool>,
    #[serde(rename="sourceContract")]
    pub source_contract: Option<SubscriptionContract>,
    pub status: Option<SubscriptionBillingCycleBillingCycleStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionBillingCycleBillingCycleStatus {
    Billed,
    Unbilled,
}

impl SubscriptionBillingCycleBillingCycleStatus {
    pub fn from_str(status: &str) -> SubscriptionBillingCycleBillingCycleStatus {
        match status {
            "BILLED" => SubscriptionBillingCycleBillingCycleStatus::Billed,
            "UNBILLED" | _ => SubscriptionBillingCycleBillingCycleStatus::Unbilled,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionBillingCycleBillingCycleStatus::Billed => String::from("BILLED"),
            SubscriptionBillingCycleBillingCycleStatus::Unbilled => String::from("UNBILLED"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingCycleEditedContract {
    pub app: Option<App>,
    #[serde(rename="appAdminUrl")]
    pub app_admin_url: Option<String>,     // The URL of the subscription contract page o the subscription app
    #[serde(rename="billingCycles")]
    pub billing_cycles: Option<SubscriptionBillingCycleConnection>,
    // ...
}

// Connections:
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingCycleConnection {
    // ...
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingAttemptsConnection {
    // ...
}

 
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionContractConnection {
   pub edges: Option<Vec<SubscriptionContractEdge>>,
   // pub nodes: Option<Vec<SubscriptionContract>>,
   #[serde(rename="pageInfo")]
   pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionContractEdge {
    pub node: Option<SubscriptionContract>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionContract {
    pub app: Option<App>,
    #[serde(rename="appAdminUrl")]
    pub app_admin_url: Option<String>,
    #[serde(rename="billingAttempts")]
    pub billing_attempts: Option<SubscriptionBillingAttemptConnection>,
    #[serde(rename="billingPolicy")]
    pub billing_policy: Option<SubscriptionBillingPolicy>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename="currencyCode")]
    pub currency_code: Option<CurrencyCode>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<Attribute>>,
    pub customer: Option<Customer>,
    #[serde(rename="customerPaymentMethod")]
    pub customer_payment_method: Option<CustomerPaymentMethod>,
    // #[serde(rename="deliveryMethod")]
    // pub delivery_method: Option<SubscriptionDeliveryMethod>, // union types
    #[serde(rename="deliveryPolicy")]
    pub delivery_policy: Option<SubscriptionDeliveryPolicy>,
    #[serde(rename="deliveryPrice")]
    pub delivery_price: Option<MoneyV2>,
    pub discounts: Option<SubscriptionManualDiscountConnection>,
    pub id: Option<String>,
    #[serde(rename="lastBillingAttemptErrorType")]
    pub last_billing_attempt_error_type: Option<SubscriptionContractLastBillingErrorType>,
    #[serde(rename="lastPaymentStatus")]
    pub last_payment_status: Option<SubscriptionContractLastPaymentStatus>,
    pub lines: Option<SubscriptionLineConnection>,
    #[serde(rename="linesCount")]
    pub lines_count: Option<Count>,
    #[serde(rename="nextBillingDate")]
    pub next_billing_date: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub orders: Option<OrderConnection>,
    #[serde(rename="originOrder")]
    pub origin_order: Option<Order>,
    #[serde(rename="revisionId")]
    pub revision_id: Option<u64>,
    pub status: Option<SubscriptionContractSubscriptionStatus>,
    #[serde(rename="updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionContractSubscriptionStatus {
    ACTIVE,
    CANCELLED,
    EXPIRED,
    FAILED,
    PAUSED,
}

impl SubscriptionContractSubscriptionStatus {
    pub fn from_str(status: &str) -> SubscriptionContractSubscriptionStatus {
        match status {
            "ACTIVE" => SubscriptionContractSubscriptionStatus::ACTIVE,
            "CANCELLED" => SubscriptionContractSubscriptionStatus::CANCELLED,
            "EXPIRED" => SubscriptionContractSubscriptionStatus::EXPIRED,
            "FAILED" => SubscriptionContractSubscriptionStatus::FAILED,
            _ => SubscriptionContractSubscriptionStatus::PAUSED,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            SubscriptionContractSubscriptionStatus::ACTIVE => String::from("ACTIVE"),
            SubscriptionContractSubscriptionStatus::CANCELLED => String::from("CANCELLED"),
            SubscriptionContractSubscriptionStatus::EXPIRED => String::from("EXPIRED"),
            SubscriptionContractSubscriptionStatus::FAILED => String::from("FAILED"),
            SubscriptionContractSubscriptionStatus::PAUSED => String::from("PAUSED"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionContractLastPaymentStatus {
    Failed,
    Succeeded,
}

impl SubscriptionContractLastPaymentStatus {
    pub fn from_str(status: &str) -> SubscriptionContractLastPaymentStatus {
        match status {
            "FAILED" => SubscriptionContractLastPaymentStatus::Failed,
            "SUCCEEDED" | _ => SubscriptionContractLastPaymentStatus::Succeeded,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionContractLastPaymentStatus::Failed => String::from("FAILED"),
            SubscriptionContractLastPaymentStatus::Succeeded => String::from("SUCCEEDED"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionContractLastBillingErrorType {
    CustomerError,
    InventoryError,
    Other,
    PaymentError,
}

impl SubscriptionContractLastBillingErrorType {
    pub fn from_str(error_type: &str) -> SubscriptionContractLastBillingErrorType {
        match error_type {
            "CUSTOMER_ERROR" => SubscriptionContractLastBillingErrorType::CustomerError,
            "INVENTORY_ERROR" => SubscriptionContractLastBillingErrorType::InventoryError,
            "PAYMENT_ERROR" => SubscriptionContractLastBillingErrorType::PaymentError,
            "OTHER" | _ => SubscriptionContractLastBillingErrorType::Other,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionContractLastBillingErrorType::CustomerError => String::from("CUSTOMER_ERROR"),
            SubscriptionContractLastBillingErrorType::InventoryError => String::from("INVENTORY_ERROR"),
            SubscriptionContractLastBillingErrorType::Other => String::from("OTHER"),
            SubscriptionContractLastBillingErrorType::PaymentError => String::from("PAYMENT_ERROR"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionManualDiscountConnection {
    pub edges: Option<Vec<SubscriptionManualDiscountEdge>>,
    // pub nodes: Option<Vec<SubscriptionManualDiscount>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionManualDiscountEdge {
    pub node: Option<SubscriptionManualDiscount>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionManualDiscount {
    #[serde(rename="entitledLines")]
    pub entitled_lines: Option<SubscriptionDiscountEntitledLines>,
    pub id: Option<String>,
    #[serde(rename="recurringCycleLimit")]
    pub recurring_cycle_limit: Option<u32>,
    #[serde(rename="rejectionReason")]
    pub rejection_reason: Option<SubscriptionDiscountRejectionReason>,
    #[serde(rename="targetType")]
    pub target_type: Option<DiscountTargetType>,
    pub title: Option<String>,
    #[serde(rename="type")]
    pub subscription_manual_discount_type: Option<DiscountType>,
    #[serde(rename="usageCount")]
    pub usage_count: Option<u32>,
    pub value: Option<SubscriptionDiscountValue>
}

// Fixed amount value type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountFixedAmountValue {
    pub amount: MoneyV2,
    #[serde(rename = "appliesOnEachInterval")]
    pub applies_on_each_interval: bool,
}

// Percentage value type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountPercentageValue {
    pub percentage: f64,
}

// Union type represented as an enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
pub enum SubscriptionDiscountValue {
    SubscriptionDiscountFixedAmountValue(SubscriptionDiscountFixedAmountValue),
    SubscriptionDiscountPercentageValue(SubscriptionDiscountPercentageValue),
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountEntitledLines {
    pub all: Option<bool>,
    pub lines: Option<SubscriptionLineConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionLineConnection {
    pub edges: Option<Vec<SubscriptionLineEdge>>,
    // pub nodes: Option<Vec<SubscriptionLine>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionLineEdge {
   pub node: Option<SubscriptionLine>,
   pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionLine {
   #[serde(rename="concatenatedOriginContract")]
   pub concatenated_origin_contract: Option<SubscriptionContract>,
   #[serde(rename="currentPrice")]
   pub current_price: Option<MoneyV2>,
   #[serde(rename="customAttributes")]
   pub custom_attributes: Option<Vec<Attribute>>,
   #[serde(rename="discountAllocations")]
   pub discount_allocations: Option<Vec<SubscriptionDiscountAllocation>>,
   pub id: Option<String>,
   #[serde(rename="lineDiscountedPrice")]
   pub line_discounted_price: Option<MoneyV2>,
   #[serde(rename="pricingPolicy")]
   pub pricing_policy: Option<SubscriptionPricingPolicy>,
   #[serde(rename="productId")]
   pub product_id: Option<String>,
   pub quantity: Option<u32>,
   #[serde(rename="requiresShipping")]
   pub requires_shipping: Option<bool>,
   #[serde(rename="sellingPlanId")]
   pub selling_plan_id: Option<String>,
   #[serde(rename="sellingPlanName")]
   pub selling_plan_name: Option<String>,
   pub sku: Option<String>,
   pub taxable: Option<bool>,
   pub title: Option<String>,
   #[serde(rename="variantId")]
   pub variant_id: Option<String>,
   #[serde(rename="variantImage")]
   pub varinat_image: Option<Image>,
   #[serde(rename="variantTitle")]
   pub variant_title: Option<String>, 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionPricingPolicy {
   #[serde(rename="basePrice")]
   pub base_price: Option<Decimal>,
   #[serde(rename="cycleDiscounts")]
   pub cycle_discounts: Option<Vec<SubscriptionCyclePriceAdjustment>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionCyclePriceAdjustment {
   #[serde(rename="adjustmentType")]
   pub adjustment_type: Option<SellingPlanPricingPolicyAdjustmentType>,
   #[serde(rename="adjustmentValue")]
   pub adjustment_value: Option<SellingPlanPricingPolicyAdjustmentValue>,
   #[serde(rename="afterCycle")]
   pub after_cycle: Option<u32>,
   #[serde(rename="computedPrice")]
   pub computed_price: Option<MoneyV2>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDiscountAllocation {
   pub amount: Option<MoneyV2>,
   pub discount: Option<SubscriptionDiscount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionAppliedCodeDiscount {
    pub id: String,
    #[serde(rename = "redeemCode")]
    pub redeem_code: String,
    #[serde(rename = "rejectionReason")]
    pub rejection_reason: Option<SubscriptionDiscountRejectionReason>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionDiscountRejectionReason {
    CurrentlyInactive,
    CustomerNotEligible,
    CustomerUsageLimitReached,
    IncompatiblePurchaseType,
    InternalError,
    NotFound,
    NoEntitledLineItems,
    NoEntitledShippingLines,
    PurchaseNotInRange,
    QuantityNotInRange,
    UsageLimitReached,
}

impl SubscriptionDiscountRejectionReason {
    pub fn from_str(reason: &str) -> SubscriptionDiscountRejectionReason {
        match reason {
            "CURRENTLY_INACTIVE" => SubscriptionDiscountRejectionReason::CurrentlyInactive,
            "CUSTOMER_NOT_ELIGIBLE" => SubscriptionDiscountRejectionReason::CustomerNotEligible,
            "CUSTOMER_USAGE_LIMIT_REACHED" => SubscriptionDiscountRejectionReason::CustomerUsageLimitReached,
            "INCOMPATIBLE_PURCHASE_TYPE" => SubscriptionDiscountRejectionReason::IncompatiblePurchaseType,
            "INTERNAL_ERROR" => SubscriptionDiscountRejectionReason::InternalError,
            "NOT_FOUND" => SubscriptionDiscountRejectionReason::NotFound,
            "NO_ENTITLED_LINE_ITEMS" => SubscriptionDiscountRejectionReason::NoEntitledLineItems,
            "NO_ENTITLED_SHIPPING_LINES" => SubscriptionDiscountRejectionReason::NoEntitledShippingLines,
            "PURCHASE_NOT_IN_RANGE" => SubscriptionDiscountRejectionReason::PurchaseNotInRange,
            "QUANTITY_NOT_IN_RANGE" => SubscriptionDiscountRejectionReason::QuantityNotInRange,
            _ => SubscriptionDiscountRejectionReason::UsageLimitReached,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionDiscountRejectionReason::CurrentlyInactive => String::from("CURRENTLY_INACTIVE"),
            SubscriptionDiscountRejectionReason::CustomerNotEligible => String::from("CUSTOMER_NOT_ELIGIBLE"),
            SubscriptionDiscountRejectionReason::CustomerUsageLimitReached => String::from("CUSTOMER_USAGE_LIMIT_REACHED"),
            SubscriptionDiscountRejectionReason::IncompatiblePurchaseType => String::from("INCOMPATIBLE_PURCHASE_TYPE"),
            SubscriptionDiscountRejectionReason::InternalError => String::from("INTERNAL_ERROR"),
            SubscriptionDiscountRejectionReason::NotFound => String::from("NOT_FOUND"),
            SubscriptionDiscountRejectionReason::NoEntitledLineItems => String::from("NO_ENTITLED_LINE_ITEMS"),
            SubscriptionDiscountRejectionReason::NoEntitledShippingLines => String::from("NO_ENTITLED_SHIPPING_LINES"),
            SubscriptionDiscountRejectionReason::PurchaseNotInRange => String::from("PURCHASE_NOT_IN_RANGE"),
            SubscriptionDiscountRejectionReason::QuantityNotInRange => String::from("QUANTITY_NOT_IN_RANGE"),
            SubscriptionDiscountRejectionReason::UsageLimitReached => String::from("USAGE_LIMIT_REACHED"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")] // This helps serde distinguish between variants (union type!)
pub enum SubscriptionDiscount {
    SubscriptionAppliedCodeDiscount(SubscriptionAppliedCodeDiscount),
    SubscriptionManualDiscount(SubscriptionManualDiscount),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryPolicy {
    pub anchors: Option<SellingPlanAnchor>,
    pub interval: Option<SellingPlanInterval>,
    #[serde(rename="intervalCount")]
    pub interval_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethod {
    // TODO! when applicable
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingAttemptConnection {
    pub edges: Option<Vec<SubscriptionBillingAttemptEdge>>,
    // pub nodes: Option<Vec<SubscriptionBillingAttempt>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingAttemptEdge {
    pub node: Option<SubscriptionBillingAttempt>,
    pub cursor: Option<String>,
}

// https://shopify.dev/docs/api/admin-graphql/2025-04/objects/SubscriptionBillingAttempt
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingAttempt {
    #[serde(rename="completedAt")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub id: Option<String>,
    #[serde(rename="indempotencyKey")]
    pub indempotency_key: Option<String>,
    // #[serde(rename="nextActionUrl")]
    // pub next_action_url: Option<String>,         // 3D Secure
    pub order: Option<Order>,
    #[serde(rename="originTime")]
    pub origin_time: Option<DateTime<Utc>>,
    #[serde(rename="paymentGroupId")]
    pub payment_group_id: Option<String>,
    #[serde(rename="paymentSessionId")]
    pub payment_session_id: Option<String>,
    #[serde(rename="processingError")]
    pub processing_error: Option<SubscriptionBillingAttemptProcessingError>,
    pub ready: Option<bool>,
    #[serde(rename="respectInventoryPolicy")]
    pub respect_inventory_policy: Option<bool>,
    #[serde(rename="subscriptionContract")]
    pub subscription_contract: Option<SubscriptionContract>,
    pub transactions: Option<OrderTransactionConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionBillingAttemptProcessingError {
    pub code: Option<SubscriptionBillingAttemptErrorCode>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionBillingAttemptErrorCode {
    AmountToSmall,
    AuthenticationError,
    BuyerCanceledPaymentMethod,
    CardNumberIncorrect,
    CustomerInvalid,
    CustomerNotFound,
    ExpiredPaymentMethod,
    FraudSuspected,
    InsufficientFunds,
    InsufficientInventory,
    InvalidCustomerBillingAgreement,
    InvalidPaymentMethod,
    InvalidShippingAddress,
    InventoryAllocationsNotFound,
    InvoiceAlreadyPaid,
    PaymentMethodDeclined,
    PaymentMethodIncompatibleWithGatewayConfig,
    PaymentMethodNotFound,
    PaymentProviderIsNotEnabled,
    PaypalErrorGeneral,
    PurchaseTypeNotSupported,
    TestMode,
    TransientError,
    UnexpectedError,
}

impl SubscriptionBillingAttemptErrorCode {
    pub fn from_str(error_code: &str) -> SubscriptionBillingAttemptErrorCode {
        match error_code {
            "AMOUNT_TO_SMALL" => SubscriptionBillingAttemptErrorCode::AmountToSmall,
            "AUTHENTICATION_ERROR" => SubscriptionBillingAttemptErrorCode::AuthenticationError,
            "BUYER_CANCELED_PAYMENT_METHOD" => SubscriptionBillingAttemptErrorCode::BuyerCanceledPaymentMethod,
            "CARD_NUMBER_INCORRECT" => SubscriptionBillingAttemptErrorCode::CardNumberIncorrect,
            "CUSTOMER_INVALID" => SubscriptionBillingAttemptErrorCode::CustomerInvalid,
            "CUSTOMER_NOT_FOUND" => SubscriptionBillingAttemptErrorCode::CustomerNotFound,
            "EXPIRED_PAYMENT_METHOD" => SubscriptionBillingAttemptErrorCode::ExpiredPaymentMethod,
            "FRAUD_SUSPECTED" => SubscriptionBillingAttemptErrorCode::FraudSuspected,
            "INSUFFICIENT_FUNDS" => SubscriptionBillingAttemptErrorCode::InsufficientFunds,
            "INSUFFICIENT_INVENTORY" => SubscriptionBillingAttemptErrorCode::InsufficientInventory,
            "INVALID_CUSTOMER_BILLING_AGREEMENT" => SubscriptionBillingAttemptErrorCode::InvalidCustomerBillingAgreement,
            "INVALID_PAYMENT_METHOD" => SubscriptionBillingAttemptErrorCode::InvalidPaymentMethod,
            "INVALID_SHIPPING_ADDRESS" => SubscriptionBillingAttemptErrorCode::InvalidShippingAddress,
            "INVENTORY_ALLOCATIONS_NOT_FOUND" => SubscriptionBillingAttemptErrorCode::InventoryAllocationsNotFound,
            "INVOICE_ALREADY_PAID" => SubscriptionBillingAttemptErrorCode::InvoiceAlreadyPaid,
            "PAYMENT_METHOD_DECLINED" => SubscriptionBillingAttemptErrorCode::PaymentMethodDeclined,
            "PAYMENT_METHOD_INCOMPATIBLE_WITH_GATEWAY_CONFIG" => SubscriptionBillingAttemptErrorCode::PaymentMethodIncompatibleWithGatewayConfig,
            "PAYMENT_METHOD_NOT_FOUND" => SubscriptionBillingAttemptErrorCode::PaymentMethodNotFound,
            "PAYMENT_PROVIDER_IS_NOT_ENABLED" => SubscriptionBillingAttemptErrorCode::PaymentProviderIsNotEnabled,
            "PAYPAL_ERROR_GENERAL" => SubscriptionBillingAttemptErrorCode::PaypalErrorGeneral,
            "PURCHASE_TYPE_NOT_SUPPORTED" => SubscriptionBillingAttemptErrorCode::PurchaseTypeNotSupported,
            "TEST_MODE" => SubscriptionBillingAttemptErrorCode::TestMode,
            "TRANSIENT_ERROR" => SubscriptionBillingAttemptErrorCode::TransientError,
            _ => SubscriptionBillingAttemptErrorCode::UnexpectedError,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionBillingAttemptErrorCode::AmountToSmall => String::from("AMOUNT_TO_SMALL"),
            SubscriptionBillingAttemptErrorCode::AuthenticationError => String::from("AUTHENTICATION_ERROR"),
            SubscriptionBillingAttemptErrorCode::BuyerCanceledPaymentMethod => String::from("BUYER_CANCELED_PAYMENT_METHOD"),
            SubscriptionBillingAttemptErrorCode::CardNumberIncorrect => String::from("CARD_NUMBER_INCORRECT"),
            SubscriptionBillingAttemptErrorCode::CustomerInvalid => String::from("CUSTOMER_INVALID"),
            SubscriptionBillingAttemptErrorCode::CustomerNotFound => String::from("CUSTOMER_NOT_FOUND"),
            SubscriptionBillingAttemptErrorCode::ExpiredPaymentMethod => String::from("EXPIRED_PAYMENT_METHOD"),
            SubscriptionBillingAttemptErrorCode::FraudSuspected => String::from("FRAUD_SUSPECTED"),
            SubscriptionBillingAttemptErrorCode::InsufficientFunds => String::from("INSUFFICIENT_FUNDS"),
            SubscriptionBillingAttemptErrorCode::InsufficientInventory => String::from("INSUFFICIENT_INVENTORY"),
            SubscriptionBillingAttemptErrorCode::InvalidCustomerBillingAgreement => String::from("INVALID_CUSTOMER_BILLING_AGREEMENT"),
            SubscriptionBillingAttemptErrorCode::InvalidPaymentMethod => String::from("INVALID_PAYMENT_METHOD"),
            SubscriptionBillingAttemptErrorCode::InvalidShippingAddress => String::from("INVALID_SHIPPING_ADDRESS"),
            SubscriptionBillingAttemptErrorCode::InventoryAllocationsNotFound => String::from("INVENTORY_ALLOCATIONS_NOT_FOUND"),
            SubscriptionBillingAttemptErrorCode::InvoiceAlreadyPaid => String::from("INVOICE_ALREADY_PAID"),
            SubscriptionBillingAttemptErrorCode::PaymentMethodDeclined => String::from("PAYMENT_METHOD_DECLINED"),
            SubscriptionBillingAttemptErrorCode::PaymentMethodIncompatibleWithGatewayConfig => String::from("PAYMENT_METHOD_INCOMPATIBLE_WITH_GATEWAY_CONFIG"),
            SubscriptionBillingAttemptErrorCode::PaymentMethodNotFound => String::from("PAYMENT_METHOD_NOT_FOUND"),
            SubscriptionBillingAttemptErrorCode::PaymentProviderIsNotEnabled => String::from("PAYMENT_PROVIDER_IS_NOT_ENABLED"),
            SubscriptionBillingAttemptErrorCode::PaypalErrorGeneral => String::from("PAYPAL_ERROR_GENERAL"),
            SubscriptionBillingAttemptErrorCode::PurchaseTypeNotSupported => String::from("PURCHASE_TYPE_NOT_SUPPORTED"),
            SubscriptionBillingAttemptErrorCode::TestMode => String::from("TEST_MODE"),
            SubscriptionBillingAttemptErrorCode::TransientError => String::from("TRANSIENT_ERROR"),
            SubscriptionBillingAttemptErrorCode::UnexpectedError => String::from("UNEXPECTED_ERROR"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionLineUpdateInput {
    #[serde(rename="currentPrice")]
    pub current_price: Option<Decimal>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<Attribute>>,
    #[serde(rename="pricingPolicy")]
    pub pricing_policy: Option<SubscriptionPricingPolicyInput>,
    #[serde(rename="productVariantId")]
    pub product_variant_id: Option<String>,
    pub quantity: Option<i32>,
    #[serde(rename="sellingPlanId")]
    pub selling_plan_id: Option<String>,
    #[serde(rename="sellingPlanName")]
    pub selling_plan_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionPricingPolicyInput {
    #[serde(rename="basePrice")]
    pub base_price: Option<Decimal>,
    #[serde(rename="cycleDiscounts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_discounts: Option<Vec<SubscriptionPricingPolicyCycleDiscountsInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionPricingPolicyCycleDiscountsInput {
    #[serde(rename="adjustmentType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_type: Option<SellingPlanPricingPolicyAdjustmentType>,
    #[serde(rename="adjustmentValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_value: Option<SellingPlanPricingPolicyValueInput>,
    #[serde(rename="afterCycle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_cycle: Option<i32>,
    #[serde(rename="computedPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_price: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SellingPlanPricingPolicyValueInput {
    #[serde(rename="fixedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_value: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

/*
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionPricingPolicyCycleDiscountsInput {
    #[serde(rename = "adjustmentType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_type: Option<SellingPlanPricingPolicyAdjustmentType>,
    #[serde(rename = "adjustmentValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_value: Option<SellingPlanPricingPolicyValueInput>,
    #[serde(rename = "afterCycle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_cycle: Option<i32>,         // cycle after which the pricing policy applies.
    #[serde(rename = "computedPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_price: Option<Decimal>,  // The computed price after the adjustments are applied.

}*/

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionContractCreateInput {
   pub contract: Option<SubscriptionDraftInput>,    // required
   #[serde(rename="currencyCode")]
   pub currency_code: Option<CurrencyCode>,
   #[serde(rename="customerId")]
   pub customer_id: Option<String>,                 // GID - So if customer needs would need to exist (non null)
   #[serde(rename="nextBillingDate")]
   pub next_billing_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionDraftInput{
   #[serde(rename="billingPolicy", skip_serializing_if = "Option::is_none")]
   pub billing_policy: Option<SubscriptionBillingPolicyInput>,
   #[serde(rename="customAttributes", skip_serializing_if = "Option::is_none")]
   pub custom_attributes: Option<Vec<Attribute>>,
   #[serde(rename="deliveryMethod", skip_serializing_if = "Option::is_none")]
   pub delivery_method: Option<SubscriptionDeliveryMethodInput>,
   #[serde(rename="deliveryPolicy", skip_serializing_if = "Option::is_none")]
   pub delivery_policy: Option<SubscriptionDeliveryPolicyInput>,
   #[serde(rename="deliveryPrice", skip_serializing_if = "Option::is_none")]
   pub delivery_price: Option<Decimal>,
   #[serde(rename="nextBillingDate", skip_serializing_if = "Option::is_none")]
   pub next_billing_date: Option<DateTime<Utc>>,
   #[serde(skip_serializing_if = "Option::is_none")]
   pub note: Option<String>,
   #[serde(rename="paymentMethodId", skip_serializing_if = "Option::is_none")]
   pub payment_method_id: Option<String>,     // required - GID
   #[serde(skip_serializing_if = "Option::is_none")]
   pub status: Option<SubscriptionContractSubscriptionStatus>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryPolicyInput{
    pub anchors: Option<Vec<SellingPlanAnchorInput>>,
    pub interval: Option<SellingPlanInterval>,
    #[serde(rename="intervalCount")]
    pub interval_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodInput{
    #[serde(rename="localDelivery")]
    pub local_delivery: Option<SubscriptionDeliveryMethodLocalDeliveryInput>,
    pub pickup: Option<SubscriptionDeliveryMethodPickupInput>,
    pub shipping: Option<SubscriptionDeliveryMethodShippingInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodShippingInput {
    pub address: Option<MailingAddressInput>,
    #[serde(rename="shippingOption")]
    pub shipping_option: Option<SubscriptionDeliveryMethodShippingOptionInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodShippingOptionInput{
    #[serde(rename="carrierServiceId")]
    pub carrier_service_id: Option<String>,
    pub code: Option<String>,
    pub decription: Option<String>,
    #[serde(rename="presentmentTitle")]
    pub presentment_title: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodPickupInput {
    #[serde(rename="pickupOption")]
    pub pickup_option: Option<SubscriptionDeliveryMethodPickupOptionInput>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodPickupOptionInput {
    pub code: Option<String>,
    pub description: Option<String>,
    #[serde(rename="locationId")]
    pub location_id: Option<String>,   // location GID
    #[serde(rename="presentmentTitle")]
    pub presentment_title: Option<String>,
    pub title: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodLocalDeliveryInput {
    pub address: Option<MailingAddressInput>,
    #[serde(rename="localDeliveryOption")]
    pub local_delivery_option: Option<SubscriptionDeliveryMethodLocalDeliveryOptionInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionDeliveryMethodLocalDeliveryOptionInput {
    pub code: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub phone: Option<String>,
    #[serde(rename="presentmentTitle")]
    pub presentment_title: Option<String>,
    pub title: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionBillingPolicyInput {
   pub anchors: Option<Vec<SellingPlanAnchorInput>>,
   pub interval: Option<SellingPlanInterval>,
   #[serde(rename="intervalCount")]
   pub interval_count: Option<u32>,
   #[serde(rename="maxCycles")]
   pub max_cycles: Option<u32>,
   #[serde(rename="minCycles")]
   pub min_cycles: Option<u32>, 
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionLineInput {
    #[serde(rename="currentPrice")]
    pub current_price: Option<Decimal>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<AttributeInput>>,
    #[serde(rename="pricingPolicy")]
    pub pricing_policy: Option<SubscriptionPricingPolicyInput>,
    #[serde(rename="productVariantId")]
    pub product_variant_id: Option<String>,
    pub quantity: Option<u32>,
    #[serde(rename="sellingPlanId")]
    pub selling_plan_id: Option<String>,
    #[serde(rename="sellingPlanName")]
    pub selling_plan_name: Option<String>,
}

