#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use crate::products::ProductInput;
use crate::variants::ProductVariant;
use crate::PageInfo;
use chrono::{DateTime, Utc};
use crate::taxes::TaxLine;
use crate::customer::Customer;
use crate::address::MailingAddress;
use crate::money::MoneyBag;
use crate::money::Decimal;
use crate::money::MoneyV2;
use crate::money::CurrencyCode;
use crate::image::Image;
use crate::JSON;
use crate::user::StaffMember;
use crate::subscriptions::Attribute;
use crate::subscriptions::SubscriptionContract;

pub mod getOrderLineItemDiscountCodes;
pub mod orderCancel;

// https://shopify.dev/docs/api/admin-graphql/2025-04/objects/Order
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub id: Option<String>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub customer: Option<Customer>,
    #[serde(rename="lineItems")]
    pub line_items: Option<LineItemConnection>,
    #[serde(rename="sourceName")]
    pub source_name: Option<String>,
    // ... other fields
    #[serde(rename="billingAddress")]
    pub billing_address: Option<MailingAddress>,
    pub name: Option<String>,
    pub note: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename="taxesIncluded")]
    pub taxes_included: Option<bool>,
    #[serde(rename="taxExempt")]
    pub tax_exempt: Option<bool>,
    #[serde(rename="taxLines")]
    pub tax_lines: Option<Vec<TaxLine>>,      // A list of all tax lines applied to line items on the order, before returns   
    #[serde(rename="totalDiscountsSet")]
    pub total_discounts_set: Option<MoneyBag>,
    #[serde(rename="totalPriceSet")]
    pub total_price_set: Option<MoneyBag>,    // Total Price of the Order, before returns. This includes taxes and discounts.
    pub test: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineItemConnection {
    pub edges: Option<Vec<LineItemEdge>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineItemEdge {
    pub node: Option<LineItem>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineItem {
    pub id: Option<String>,
    pub title: Option<String>,
    pub product: Option<ProductInput>,
    pub variant: Option<ProductVariant>,
    pub quantity: Option<u64>,
    #[serde(rename="taxLines")]
    pub tax_lines: Option<Vec<TaxLine>>,
    #[serde(rename="originalTotalSet")]
    pub original_total_set: Option<MoneyBag>,
    #[serde(rename="discountedTotalSet")]
    pub discounted_total_set: Option<MoneyBag>,
    pub name: Option<String>,
    pub sku: Option<String>,
    #[serde(rename="requiresShipping")]
    pub requires_shipping: Option<bool>,
    #[serde(rename="customAttributes")]
    pub custom_attributes: Option<Vec<Attribute>>,
    pub contract: Option<SubscriptionContract>,
}

// Connections: 
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderConnection {
    pub edges: Option<Vec<OrderEdge>>,
    // pub nodes: Option<Vec<Order>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderEdge {
    pub node: Option<Order>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderTransactionConnection {
    pub edges: Option<Vec<OrderTransactionEdge>>,
    // pub nodes: Option<Vec<OrderTransaction>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderTransactionEdge {
    pub node: Option<OrderTransaction>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderTransaction {
    #[serde(rename="accountNumber")]
    pub account_number: Option<String>,
    #[serde(rename="amountRoundingSet")]
    pub amount_rounding_set: Option<MoneyBag>,
    #[serde(rename="amountSet")]
    pub amount_set: Option<MoneyBag>,
    #[serde(rename="authorizationCode")]
    pub authorization_code: Option<String>,
    #[serde(rename="authorizationExpiresAt")]
    pub authorization_expires_at: Option<DateTime<Utc>>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename="errorCode")]
    pub error_code: Option<OrderTransactionErrorCode>,
    pub fees: Option<Vec<TransactionFee>>,
    #[serde(rename="formattedGateway")]
    pub formatted_gateway: Option<String>,
    pub gateway: Option<String>,
    pub id: Option<String>,
    pub kind: Option<OrderTransactionKind>,
    #[serde(rename="manuallyCapturable")]
    pub manaully_capturable: Option<bool>,
    #[serde(rename="manualPaymentGateway")]
    pub manual_payment_gateway: Option<bool>,
    #[serde(rename="maximumRefundableV2")]
    pub maximum_refundable_v2: Option<MoneyV2>,
    #[serde(rename="multiCapturable")]
    pub multi_capturable: Option<bool>,
    pub order: Option<Order>,
    #[serde(rename="parentTransaction")]
    pub parent_transaction: Option<Box<OrderTransaction>>,  // recursive without indirection use box (on heap)
    #[serde(rename="paymentDetails")]
    pub payment_details: Option<PaymentDetails>,
    #[serde(rename="paymentIcon")]
    pub payment_icon: Option<Image>,
    #[serde(rename="paymentId")]
    pub payment_id: Option<String>,
    #[serde(rename="processedAt")]
    pub processed_at: Option<DateTime<Utc>>,
    #[serde(rename="recieptJson")]
    pub reciept_json: Option<JSON>,
    #[serde(rename="settlementCurrency")]
    pub settlement_currency: Option<CurrencyCode>,
    #[serde(rename="settlementCurrencyRate")]
    pub settlement_currency_rate: Option<Decimal>,
    // #[serde(rename="shopifyPaymentSet")]
    // pub shopify_payment_set: Option<ShopifyPaymentTransactionSet>
    pub status: Option<OrderTransactionStatus>,
    pub test: Option<bool>,
    #[serde(rename="totalUnsettledSet")]
    pub total_unsettled_set: Option<MoneyBag>,
    pub user: Option<StaffMember>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTransactionStatus {
    AwaitingResponse,
    Error,
    Failure,
    Pending,
    Success,
    Unknown,
}

impl OrderTransactionStatus {
    pub fn from_str(status: &str) -> OrderTransactionStatus {
        match status.to_uppercase().as_str() {
            "AWAITING_RESPONSE" => OrderTransactionStatus::AwaitingResponse,
            "ERROR" => OrderTransactionStatus::Error,
            "FAILURE" => OrderTransactionStatus::Failure,
            "PENDING" => OrderTransactionStatus::Pending,
            "SUCCESS" => OrderTransactionStatus::Success,
            _ => OrderTransactionStatus::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderTransactionStatus::AwaitingResponse => String::from("AWAITING_RESPONSE"),
            OrderTransactionStatus::Error => String::from("ERROR"),
            OrderTransactionStatus::Failure => String::from("FAILURE"),
            OrderTransactionStatus::Pending => String::from("PENDING"),
            OrderTransactionStatus::Success => String::from("SUCCESS"),
            OrderTransactionStatus::Unknown => String::from("UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardPaymentDetails {
    #[serde(rename = "avsResultCode")]
    pub avs_result_code: Option<String>,
    pub bin: Option<String>,
    pub company: Option<String>,
    #[serde(rename = "cvvResultCode")]
    pub cvv_result_code: Option<String>,
    #[serde(rename = "expirationMonth")]
    pub expiration_month: Option<i32>,
    #[serde(rename = "expirationYear")]
    pub expiration_year: Option<i32>,
    pub name: Option<String>,
    pub number: Option<String>,
    #[serde(rename = "paymentMethodName")]
    pub payment_method_name: Option<String>,
    pub wallet: Option<DigitalWallet>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DigitalWallet {
    AndroidPay,
    ApplePay,
    GooglePay,
    ShopifyPay,
}

impl DigitalWallet {
    pub fn from_str(wallet: &str) -> DigitalWallet {
        match wallet {
            "ANDROID_PAY" => DigitalWallet::AndroidPay,
            "APPLE_PAY" => DigitalWallet::ApplePay,
            "GOOGLE_PAY" => DigitalWallet::GooglePay,
            "SHOPIFY_PAY" | _ => DigitalWallet::ShopifyPay,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DigitalWallet::AndroidPay => String::from("ANDROID_PAY"),
            DigitalWallet::ApplePay => String::from("APPLE_PAY"),
            DigitalWallet::GooglePay => String::from("GOOGLE_PAY"),
            DigitalWallet::ShopifyPay => String::from("SHOPIFY_PAY"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalPaymentMethodsPaymentDetails {
    #[serde(rename="paymentDescriptor")]
    pub payment_descriptor: Option<String>,
    #[serde(rename="paymentMethodName")]
    pub payment_method_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopPayInstallmentsPaymentDetails {
    #[serde(rename = "paymentMethodName")]
    pub payment_method_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
pub enum PaymentDetails {
    CardPaymentDetails(CardPaymentDetails),
    LocalPaymentMethodsPaymentDetails(LocalPaymentMethodsPaymentDetails),
    ShopPayInstallmentsPaymentDetails(ShopPayInstallmentsPaymentDetails),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTransactionKind {
    Authorization,
    Capture,
    Change,
    EmvAuthorization,
    Refund,
    Sale,
    SuggestedRefund,
    Void,
}

impl OrderTransactionKind {
    pub fn from_str(kind: &str) -> OrderTransactionKind {
        match kind.to_uppercase().as_str() {
            "AUTHORIZATION" => OrderTransactionKind::Authorization,
            "CAPTURE" => OrderTransactionKind::Capture,
            "CHANGE" => OrderTransactionKind::Change,
            "EMV_AUTHORIZATION" => OrderTransactionKind::EmvAuthorization,
            "REFUND" => OrderTransactionKind::Refund,
            "SALE" => OrderTransactionKind::Sale,
            "SUGGESTED_REFUND" => OrderTransactionKind::SuggestedRefund,
            "VOID" | _ => OrderTransactionKind::Void,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderTransactionKind::Authorization => String::from("AUTHORIZATION"),
            OrderTransactionKind::Capture => String::from("CAPTURE"),
            OrderTransactionKind::Change => String::from("CHANGE"),
            OrderTransactionKind::EmvAuthorization => String::from("EMV_AUTHORIZATION"),
            OrderTransactionKind::Refund => String::from("REFUND"),
            OrderTransactionKind::Sale => String::from("SALE"),
            OrderTransactionKind::SuggestedRefund => String::from("SUGGESTED_REFUND"),
            OrderTransactionKind::Void => String::from("VOID"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionFee {
    pub amount: Option<MoneyV2>,
    #[serde(rename="flatFee")]
    pub flat_fee: Option<MoneyV2>,
    #[serde(rename="flatFeeName")]
    pub flat_fee_name: Option<String>,
    pub id: Option<String>,
    pub rate: Option<Decimal>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OrderTransactionErrorCode {
    AmazonPaymentsInvalidPaymentMethod,
    AmazonPaymentsMaxAmountCharged,
    AmazonPaymentsMaxAmountRefunded,
    AmazonPaymentsMaxAuthorizationsCaptured,
    AmazonPaymentsMaxRefundsProcessed,
    AmazonPaymentsOrderReferenceCanceled,
    AmazonPaymentsStale,
    CallIssuer,
    CardDeclined,
    ConfigError,
    ExpiredCard,
    GenericError,
    IncorrectAddress,
    IncorrectCvc,
    IncorrectNumber,
    IncorrectPin,
    IncorrectZip,
    InvalidAmount,
    InvalidCountry,
    InvalidCvc,
    InvalidExpiryDate,
    InvalidNumber,
    PaymentMethodUnavailable,
    PickUpCard,
    ProcessingError,
    TestModeLiveCard,
    UnsupportedFeature,
}

impl OrderTransactionErrorCode {
    pub fn from_str(error_code: &str) -> OrderTransactionErrorCode {
        match error_code {
            "AMAZON_PAYMENTS_INVALID_PAYMENT_METHOD" => OrderTransactionErrorCode::AmazonPaymentsInvalidPaymentMethod,
            "AMAZON_PAYMENTS_MAX_AMOUNT_CHARGED" => OrderTransactionErrorCode::AmazonPaymentsMaxAmountCharged,
            "AMAZON_PAYMENTS_MAX_AMOUNT_REFUNDED" => OrderTransactionErrorCode::AmazonPaymentsMaxAmountRefunded,
            "AMAZON_PAYMENTS_MAX_AUTHORIZATIONS_CAPTURED" => OrderTransactionErrorCode::AmazonPaymentsMaxAuthorizationsCaptured,
            "AMAZON_PAYMENTS_MAX_REFUNDS_PROCESSED" => OrderTransactionErrorCode::AmazonPaymentsMaxRefundsProcessed,
            "AMAZON_PAYMENTS_ORDER_REFERENCE_CANCELED" => OrderTransactionErrorCode::AmazonPaymentsOrderReferenceCanceled,
            "AMAZON_PAYMENTS_STALE" => OrderTransactionErrorCode::AmazonPaymentsStale,
            "CALL_ISSUER" => OrderTransactionErrorCode::CallIssuer,
            "CARD_DECLINED" => OrderTransactionErrorCode::CardDeclined,
            "CONFIG_ERROR" => OrderTransactionErrorCode::ConfigError,
            "EXPIRED_CARD" => OrderTransactionErrorCode::ExpiredCard,
            "GENERIC_ERROR" => OrderTransactionErrorCode::GenericError,
            "INCORRECT_ADDRESS" => OrderTransactionErrorCode::IncorrectAddress,
            "INCORRECT_CVC" => OrderTransactionErrorCode::IncorrectCvc,
            "INCORRECT_NUMBER" => OrderTransactionErrorCode::IncorrectNumber,
            "INCORRECT_PIN" => OrderTransactionErrorCode::IncorrectPin,
            "INCORRECT_ZIP" => OrderTransactionErrorCode::IncorrectZip,
            "INVALID_AMOUNT" => OrderTransactionErrorCode::InvalidAmount,
            "INVALID_COUNTRY" => OrderTransactionErrorCode::InvalidCountry,
            "INVALID_CVC" => OrderTransactionErrorCode::InvalidCvc,
            "INVALID_EXPIRY_DATE" => OrderTransactionErrorCode::InvalidExpiryDate,
            "INVALID_NUMBER" => OrderTransactionErrorCode::InvalidNumber,
            "PAYMENT_METHOD_UNAVAILABLE" => OrderTransactionErrorCode::PaymentMethodUnavailable,
            "PICK_UP_CARD" => OrderTransactionErrorCode::PickUpCard,
            "PROCESSING_ERROR" => OrderTransactionErrorCode::ProcessingError,
            "TEST_MODE_LIVE_CARD" => OrderTransactionErrorCode::TestModeLiveCard,
            _ => OrderTransactionErrorCode::UnsupportedFeature,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderTransactionErrorCode::AmazonPaymentsInvalidPaymentMethod => String::from("AMAZON_PAYMENTS_INVALID_PAYMENT_METHOD"),
            OrderTransactionErrorCode::AmazonPaymentsMaxAmountCharged => String::from("AMAZON_PAYMENTS_MAX_AMOUNT_CHARGED"),
            OrderTransactionErrorCode::AmazonPaymentsMaxAmountRefunded => String::from("AMAZON_PAYMENTS_MAX_AMOUNT_REFUNDED"),
            OrderTransactionErrorCode::AmazonPaymentsMaxAuthorizationsCaptured => String::from("AMAZON_PAYMENTS_MAX_AUTHORIZATIONS_CAPTURED"),
            OrderTransactionErrorCode::AmazonPaymentsMaxRefundsProcessed => String::from("AMAZON_PAYMENTS_MAX_REFUNDS_PROCESSED"),
            OrderTransactionErrorCode::AmazonPaymentsOrderReferenceCanceled => String::from("AMAZON_PAYMENTS_ORDER_REFERENCE_CANCELED"),
            OrderTransactionErrorCode::AmazonPaymentsStale => String::from("AMAZON_PAYMENTS_STALE"),
            OrderTransactionErrorCode::CallIssuer => String::from("CALL_ISSUER"),
            OrderTransactionErrorCode::CardDeclined => String::from("CARD_DECLINED"),
            OrderTransactionErrorCode::ConfigError => String::from("CONFIG_ERROR"),
            OrderTransactionErrorCode::ExpiredCard => String::from("EXPIRED_CARD"),
            OrderTransactionErrorCode::GenericError => String::from("GENERIC_ERROR"),
            OrderTransactionErrorCode::IncorrectAddress => String::from("INCORRECT_ADDRESS"),
            OrderTransactionErrorCode::IncorrectCvc => String::from("INCORRECT_CVC"),
            OrderTransactionErrorCode::IncorrectNumber => String::from("INCORRECT_NUMBER"),
            OrderTransactionErrorCode::IncorrectPin => String::from("INCORRECT_PIN"),
            OrderTransactionErrorCode::IncorrectZip => String::from("INCORRECT_ZIP"),
            OrderTransactionErrorCode::InvalidAmount => String::from("INVALID_AMOUNT"),
            OrderTransactionErrorCode::InvalidCountry => String::from("INVALID_COUNTRY"),
            OrderTransactionErrorCode::InvalidCvc => String::from("INVALID_CVC"),
            OrderTransactionErrorCode::InvalidExpiryDate => String::from("INVALID_EXPIRY_DATE"),
            OrderTransactionErrorCode::InvalidNumber => String::from("INVALID_NUMBER"),
            OrderTransactionErrorCode::PaymentMethodUnavailable => String::from("PAYMENT_METHOD_UNAVAILABLE"),
            OrderTransactionErrorCode::PickUpCard => String::from("PICK_UP_CARD"),
            OrderTransactionErrorCode::ProcessingError => String::from("PROCESSING_ERROR"),
            OrderTransactionErrorCode::TestModeLiveCard => String::from("TEST_MODE_LIVE_CARD"),
            OrderTransactionErrorCode::UnsupportedFeature => String::from("UNSUPPORTED_FEATURE"),
        }
    }
}


