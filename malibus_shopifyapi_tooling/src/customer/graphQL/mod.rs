#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use chrono::prelude::*;
use std::fmt::Debug;
use std::fmt;
use serde::de::{self, Visitor};
use serde_json::json;
use crate::error::UserError;
use crate::metafields::MetafieldInput;

pub mod customerCreate;
pub mod customerUpdate;
pub mod getCustomerInfo;
pub mod getCustomerPaymentMethods;      // We need to get the Payment Method ID before we can Update
pub mod customerPaymentUpdateEmail;     // if we need the customer to update their Payment Method

// https://shopify.dev/docs/api/admin-graphql/2024-07/input-objects/CustomerInput
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerInput {
    pub addresses: Option<Vec<MailingAddressInput>>,
    pub email: Option<String>,
    #[serde(rename="emailMarketingConsent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_marketing_consent: Option<CustomerEmailMarketingConsentInput>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub locale: Option<String>,
    pub metafields: Option<Vec<MetafieldInput>>,
    pub note: Option<String>,
    pub phone: Option<String>,
    #[serde(rename="smsMarketingConsent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_marketing_consent: Option<CustomerSmsMarketingConsentInput>,
    pub tags: Option<Vec<String>>,   // Can be an array or a comma-separated list. ["tag1", "tag2"] or "tag1, tag2, tag3"
    #[serde(rename="taxExempt")]
    pub tax_exempt: Option<bool>,
    #[serde(rename="taxExemptions")]
    pub tax_exemptions: Option<Vec<TaxExemption>>
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum TaxExemption {
    CA_BC_COMMERCIAL_FISHERY_EXEMPTION,
    CA_BC_CONTRACTOR_EXEMPTION,
    CA_BC_PRODUCTION_AND_MACHINERY_EXEMPTION,
    CA_BC_RESELLER_EXEMPTION,
    CA_BC_SUB_CONTRACTOR_EXEMPTION,
    CA_DIPLOMAT_EXEMPTION,
    CA_MB_COMMERCIAL_FISHERY_EXEMPTION,
    CA_MB_FARMER_EXEMPTION,
    CA_MB_RESELLER_EXEMPTION,
    CA_NS_COMMERCIAL_FISHERY_EXEMPTION,
    CA_NS_FARMER_EXEMPTION,
    CA_ON_PURCHASE_EXEMPTION,
    CA_PE_COMMERCIAL_FISHERY_EXEMPTION,
    CA_SK_COMMERCIAL_FISHERY_EXEMPTION,
    CA_SK_CONTRACTOR_EXEMPTION,
    CA_SK_FARMER_EXEMPTION,
    CA_SK_PRODUCTION_AND_MACHINERY_EXEMPTION,
    CA_SK_RESELLER_EXEMPTION,
    CA_SK_SUB_CONTRACTOR_EXEMPTION,
    CA_STATUS_CARD_EXEMPTION,
    EU_REVERSE_CHARGE_EXEMPTION_RULE,
    US_AK_RESELLER_EXEMPTION,
    US_AL_RESELLER_EXEMPTION,
    US_AR_RESELLER_EXEMPTION,
    US_AZ_RESELLER_EXEMPTION,
    US_CA_RESELLER_EXEMPTION,
    US_CO_RESELLER_EXEMPTION,
    US_CT_RESELLER_EXEMPTION,
    US_DC_RESELLER_EXEMPTION,
    US_DE_RESELLER_EXEMPTION,
    US_FL_RESELLER_EXEMPTION,
    US_GA_RESELLER_EXEMPTION,
    US_HI_RESELLER_EXEMPTION,
    US_IA_RESELLER_EXEMPTION,
    US_ID_RESELLER_EXEMPTION,
    US_IL_RESELLER_EXEMPTION,
    US_IN_RESELLER_EXEMPTION,
    US_KS_RESELLER_EXEMPTION,
    US_KY_RESELLER_EXEMPTION,
    US_LA_RESELLER_EXEMPTION,
    US_MA_RESELLER_EXEMPTION,
    US_MD_RESELLER_EXEMPTION,
    US_ME_RESELLER_EXEMPTION,
    US_MI_RESELLER_EXEMPTION,
    US_MN_RESELLER_EXEMPTION,
    US_MO_RESELLER_EXEMPTION,
    US_MS_RESELLER_EXEMPTION,
    US_MT_RESELLER_EXEMPTION,
    US_NC_RESELLER_EXEMPTION,
    US_ND_RESELLER_EXEMPTION,
    US_NE_RESELLER_EXEMPTION,
    US_NH_RESELLER_EXEMPTION,
    US_NJ_RESELLER_EXEMPTION,
    US_NM_RESELLER_EXEMPTION,
    US_NV_RESELLER_EXEMPTION,
    US_NY_RESELLER_EXEMPTION,
    US_OH_RESELLER_EXEMPTION,
    US_OK_RESELLER_EXEMPTION,
    US_OR_RESELLER_EXEMPTION,
    US_PA_RESELLER_EXEMPTION,
    US_RI_RESELLER_EXEMPTION,
    US_SC_RESELLER_EXEMPTION,
    US_SD_RESELLER_EXEMPTION,
    US_TN_RESELLER_EXEMPTION,
    US_TX_RESELLER_EXEMPTION,
    US_UT_RESELLER_EXEMPTION,
    US_VA_RESELLER_EXEMPTION,
    US_VT_RESELLER_EXEMPTION,
    US_WA_RESELLER_EXEMPTION,
    US_WI_RESELLER_EXEMPTION,
    US_WV_RESELLER_EXEMPTION,
    US_WY_RESELLER_EXEMPTION,
}


impl TaxExemption {
        pub fn from_str(code: &str) -> TaxExemption {
            match code {
                "CA_BC_COMMERCIAL_FISHERY_EXEMPTION" => TaxExemption::CA_BC_COMMERCIAL_FISHERY_EXEMPTION,
                "CA_BC_CONTRACTOR_EXEMPTION" => TaxExemption::CA_BC_CONTRACTOR_EXEMPTION,
                "CA_BC_PRODUCTION_AND_MACHINERY_EXEMPTION" => TaxExemption::CA_BC_PRODUCTION_AND_MACHINERY_EXEMPTION,
                "CA_BC_RESELLER_EXEMPTION" => TaxExemption::CA_BC_RESELLER_EXEMPTION,
                "CA_BC_SUB_CONTRACTOR_EXEMPTION" => TaxExemption::CA_BC_SUB_CONTRACTOR_EXEMPTION,
                "CA_DIPLOMAT_EXEMPTION" => TaxExemption::CA_DIPLOMAT_EXEMPTION,
                "CA_MB_COMMERCIAL_FISHERY_EXEMPTION" => TaxExemption::CA_MB_COMMERCIAL_FISHERY_EXEMPTION,
                "CA_MB_FARMER_EXEMPTION" => TaxExemption::CA_MB_FARMER_EXEMPTION,
                "CA_MB_RESELLER_EXEMPTION" => TaxExemption::CA_MB_RESELLER_EXEMPTION,
                "CA_NS_COMMERCIAL_FISHERY_EXEMPTION" => TaxExemption::CA_NS_COMMERCIAL_FISHERY_EXEMPTION,
                "CA_NS_FARMER_EXEMPTION" => TaxExemption::CA_NS_FARMER_EXEMPTION,
                "CA_ON_PURCHASE_EXEMPTION" => TaxExemption::CA_ON_PURCHASE_EXEMPTION,
                "CA_PE_COMMERCIAL_FISHERY_EXEMPTION" => TaxExemption::CA_PE_COMMERCIAL_FISHERY_EXEMPTION,
                "CA_SK_COMMERCIAL_FISHERY_EXEMPTION" => TaxExemption::CA_SK_COMMERCIAL_FISHERY_EXEMPTION,
                "CA_SK_CONTRACTOR_EXEMPTION" => TaxExemption::CA_SK_CONTRACTOR_EXEMPTION,
                "CA_SK_FARMER_EXEMPTION" => TaxExemption::CA_SK_FARMER_EXEMPTION,
                "CA_SK_PRODUCTION_AND_MACHINERY_EXEMPTION" => TaxExemption::CA_SK_PRODUCTION_AND_MACHINERY_EXEMPTION,
                "CA_SK_RESELLER_EXEMPTION" => TaxExemption::CA_SK_RESELLER_EXEMPTION,
                "CA_SK_SUB_CONTRACTOR_EXEMPTION" => TaxExemption::CA_SK_SUB_CONTRACTOR_EXEMPTION,
                "CA_STATUS_CARD_EXEMPTION" => TaxExemption::CA_STATUS_CARD_EXEMPTION,
                "EU_REVERSE_CHARGE_EXEMPTION_RULE" => TaxExemption::EU_REVERSE_CHARGE_EXEMPTION_RULE,
                "US_AK_RESELLER_EXEMPTION" => TaxExemption::US_AK_RESELLER_EXEMPTION,
                "US_AL_RESELLER_EXEMPTION" => TaxExemption::US_AL_RESELLER_EXEMPTION,
                "US_AR_RESELLER_EXEMPTION" => TaxExemption::US_AR_RESELLER_EXEMPTION,
                "US_AZ_RESELLER_EXEMPTION" => TaxExemption::US_AZ_RESELLER_EXEMPTION,
                "US_CA_RESELLER_EXEMPTION" => TaxExemption::US_CA_RESELLER_EXEMPTION,
                "US_CO_RESELLER_EXEMPTION" => TaxExemption::US_CO_RESELLER_EXEMPTION,
                "US_CT_RESELLER_EXEMPTION" => TaxExemption::US_CT_RESELLER_EXEMPTION,
                "US_DC_RESELLER_EXEMPTION" => TaxExemption::US_DC_RESELLER_EXEMPTION,
                "US_DE_RESELLER_EXEMPTION" => TaxExemption::US_DE_RESELLER_EXEMPTION,
                "US_FL_RESELLER_EXEMPTION" => TaxExemption::US_FL_RESELLER_EXEMPTION,
                "US_GA_RESELLER_EXEMPTION" => TaxExemption::US_GA_RESELLER_EXEMPTION,
                "US_HI_RESELLER_EXEMPTION" => TaxExemption::US_HI_RESELLER_EXEMPTION,
                "US_IA_RESELLER_EXEMPTION" => TaxExemption::US_IA_RESELLER_EXEMPTION,
                "US_ID_RESELLER_EXEMPTION" => TaxExemption::US_ID_RESELLER_EXEMPTION,
                "US_IL_RESELLER_EXEMPTION" => TaxExemption::US_IL_RESELLER_EXEMPTION,
                "US_IN_RESELLER_EXEMPTION" => TaxExemption::US_IN_RESELLER_EXEMPTION,
                "US_KS_RESELLER_EXEMPTION" => TaxExemption::US_KS_RESELLER_EXEMPTION,
                "US_KY_RESELLER_EXEMPTION" => TaxExemption::US_KY_RESELLER_EXEMPTION,
                "US_LA_RESELLER_EXEMPTION" => TaxExemption::US_LA_RESELLER_EXEMPTION,
                "US_MA_RESELLER_EXEMPTION" => TaxExemption::US_MA_RESELLER_EXEMPTION,
                "US_MD_RESELLER_EXEMPTION" => TaxExemption::US_MD_RESELLER_EXEMPTION,
                "US_ME_RESELLER_EXEMPTION" => TaxExemption::US_ME_RESELLER_EXEMPTION,
                "US_MI_RESELLER_EXEMPTION" => TaxExemption::US_MI_RESELLER_EXEMPTION,
                "US_MN_RESELLER_EXEMPTION" => TaxExemption::US_MN_RESELLER_EXEMPTION,
                "US_MO_RESELLER_EXEMPTION" => TaxExemption::US_MO_RESELLER_EXEMPTION,
                "US_MS_RESELLER_EXEMPTION" => TaxExemption::US_MS_RESELLER_EXEMPTION,
                "US_MT_RESELLER_EXEMPTION" => TaxExemption::US_MT_RESELLER_EXEMPTION,
                "US_NC_RESELLER_EXEMPTION" => TaxExemption::US_NC_RESELLER_EXEMPTION,
                "US_ND_RESELLER_EXEMPTION" => TaxExemption::US_ND_RESELLER_EXEMPTION,
                "US_NE_RESELLER_EXEMPTION" => TaxExemption::US_NE_RESELLER_EXEMPTION,
                "US_NH_RESELLER_EXEMPTION" => TaxExemption::US_NH_RESELLER_EXEMPTION,
                "US_NJ_RESELLER_EXEMPTION" => TaxExemption::US_NJ_RESELLER_EXEMPTION,
                "US_NM_RESELLER_EXEMPTION" => TaxExemption::US_NM_RESELLER_EXEMPTION,
                "US_NV_RESELLER_EXEMPTION" => TaxExemption::US_NV_RESELLER_EXEMPTION,
                "US_NY_RESELLER_EXEMPTION" => TaxExemption::US_NY_RESELLER_EXEMPTION,
                "US_OH_RESELLER_EXEMPTION" => TaxExemption::US_OH_RESELLER_EXEMPTION,
                "US_OK_RESELLER_EXEMPTION" => TaxExemption::US_OK_RESELLER_EXEMPTION,
                "US_OR_RESELLER_EXEMPTION" => TaxExemption::US_OR_RESELLER_EXEMPTION,
                "US_PA_RESELLER_EXEMPTION" => TaxExemption::US_PA_RESELLER_EXEMPTION,
                "US_RI_RESELLER_EXEMPTION" => TaxExemption::US_RI_RESELLER_EXEMPTION,
                "US_SC_RESELLER_EXEMPTION" => TaxExemption::US_SC_RESELLER_EXEMPTION,
                "US_SD_RESELLER_EXEMPTION" => TaxExemption::US_SD_RESELLER_EXEMPTION,
                "US_TN_RESELLER_EXEMPTION" => TaxExemption::US_TN_RESELLER_EXEMPTION,
                "US_TX_RESELLER_EXEMPTION" => TaxExemption::US_TX_RESELLER_EXEMPTION,
                "US_UT_RESELLER_EXEMPTION" => TaxExemption::US_UT_RESELLER_EXEMPTION,
                "US_VA_RESELLER_EXEMPTION" => TaxExemption::US_VA_RESELLER_EXEMPTION,
                "US_VT_RESELLER_EXEMPTION" => TaxExemption::US_VT_RESELLER_EXEMPTION,
                "US_WA_RESELLER_EXEMPTION" => TaxExemption::US_WA_RESELLER_EXEMPTION,
                "US_WI_RESELLER_EXEMPTION" => TaxExemption::US_WI_RESELLER_EXEMPTION,
                "US_WV_RESELLER_EXEMPTION" => TaxExemption::US_WV_RESELLER_EXEMPTION,
                "US_WY_RESELLER_EXEMPTION" => TaxExemption::US_WY_RESELLER_EXEMPTION,
                _ => panic!("Invalid TaxExemption code"),
            }
    }
    
pub fn to_string(&self) -> String {
        match self {
            TaxExemption::CA_BC_COMMERCIAL_FISHERY_EXEMPTION => String::from("CA_BC_COMMERCIAL_FISHERY_EXEMPTION"),
            TaxExemption::CA_BC_CONTRACTOR_EXEMPTION => String::from("CA_BC_CONTRACTOR_EXEMPTION"),
            TaxExemption::CA_BC_PRODUCTION_AND_MACHINERY_EXEMPTION => String::from("CA_BC_PRODUCTION_AND_MACHINERY_EXEMPTION"),
            TaxExemption::CA_BC_RESELLER_EXEMPTION => String::from("CA_BC_RESELLER_EXEMPTION"),
            TaxExemption::CA_BC_SUB_CONTRACTOR_EXEMPTION => String::from("CA_BC_SUB_CONTRACTOR_EXEMPTION"),
            TaxExemption::CA_DIPLOMAT_EXEMPTION => String::from("CA_DIPLOMAT_EXEMPTION"),
            TaxExemption::CA_MB_COMMERCIAL_FISHERY_EXEMPTION => String::from("CA_MB_COMMERCIAL_FISHERY_EXEMPTION"),
            TaxExemption::CA_MB_FARMER_EXEMPTION => String::from("CA_MB_FARMER_EXEMPTION"),
            TaxExemption::CA_MB_RESELLER_EXEMPTION => String::from("CA_MB_RESELLER_EXEMPTION"),
            TaxExemption::CA_NS_COMMERCIAL_FISHERY_EXEMPTION => String::from("CA_NS_COMMERCIAL_FISHERY_EXEMPTION"),
            TaxExemption::CA_NS_FARMER_EXEMPTION => String::from("CA_NS_FARMER_EXEMPTION"),
            TaxExemption::CA_ON_PURCHASE_EXEMPTION => String::from("CA_ON_PURCHASE_EXEMPTION"),
            TaxExemption::CA_PE_COMMERCIAL_FISHERY_EXEMPTION => String::from("CA_PE_COMMERCIAL_FISHERY_EXEMPTION"),
            TaxExemption::CA_SK_COMMERCIAL_FISHERY_EXEMPTION => String::from("CA_SK_COMMERCIAL_FISHERY_EXEMPTION"),
            TaxExemption::CA_SK_CONTRACTOR_EXEMPTION => String::from("CA_SK_CONTRACTOR_EXEMPTION"),
            TaxExemption::CA_SK_FARMER_EXEMPTION => String::from("CA_SK_FARMER_EXEMPTION"),
            TaxExemption::CA_SK_PRODUCTION_AND_MACHINERY_EXEMPTION => String::from("CA_SK_PRODUCTION_AND_MACHINERY_EXEMPTION"),
            TaxExemption::CA_SK_RESELLER_EXEMPTION => String::from("CA_SK_RESELLER_EXEMPTION"),
            TaxExemption::CA_SK_SUB_CONTRACTOR_EXEMPTION => String::from("CA_SK_SUB_CONTRACTOR_EXEMPTION"),
            TaxExemption::CA_STATUS_CARD_EXEMPTION => String::from("CA_STATUS_CARD_EXEMPTION"),
            TaxExemption::EU_REVERSE_CHARGE_EXEMPTION_RULE => String::from("EU_REVERSE_CHARGE_EXEMPTION_RULE"),
            TaxExemption::US_AK_RESELLER_EXEMPTION => String::from("US_AK_RESELLER_EXEMPTION"),
            TaxExemption::US_AL_RESELLER_EXEMPTION => String::from("US_AL_RESELLER_EXEMPTION"),
            TaxExemption::US_AR_RESELLER_EXEMPTION => String::from("US_AR_RESELLER_EXEMPTION"),
            TaxExemption::US_AZ_RESELLER_EXEMPTION => String::from("US_AZ_RESELLER_EXEMPTION"),
            TaxExemption::US_CA_RESELLER_EXEMPTION => String::from("US_CA_RESELLER_EXEMPTION"),
            TaxExemption::US_CO_RESELLER_EXEMPTION => String::from("US_CO_RESELLER_EXEMPTION"),
            TaxExemption::US_CT_RESELLER_EXEMPTION => String::from("US_CT_RESELLER_EXEMPTION"),
            TaxExemption::US_DC_RESELLER_EXEMPTION => String::from("US_DC_RESELLER_EXEMPTION"),
            TaxExemption::US_DE_RESELLER_EXEMPTION => String::from("US_DE_RESELLER_EXEMPTION"),
            TaxExemption::US_FL_RESELLER_EXEMPTION => String::from("US_FL_RESELLER_EXEMPTION"),
            TaxExemption::US_GA_RESELLER_EXEMPTION => String::from("US_GA_RESELLER_EXEMPTION"),
            TaxExemption::US_HI_RESELLER_EXEMPTION => String::from("US_HI_RESELLER_EXEMPTION"),
            TaxExemption::US_IA_RESELLER_EXEMPTION => String::from("US_IA_RESELLER_EXEMPTION"),
            TaxExemption::US_ID_RESELLER_EXEMPTION => String::from("US_ID_RESELLER_EXEMPTION"),
            TaxExemption::US_IL_RESELLER_EXEMPTION => String::from("US_IL_RESELLER_EXEMPTION"),
            TaxExemption::US_IN_RESELLER_EXEMPTION => String::from("US_IN_RESELLER_EXEMPTION"),
            TaxExemption::US_KS_RESELLER_EXEMPTION => String::from("US_KS_RESELLER_EXEMPTION"),
            TaxExemption::US_KY_RESELLER_EXEMPTION => String::from("US_KY_RESELLER_EXEMPTION"),
            TaxExemption::US_LA_RESELLER_EXEMPTION => String::from("US_LA_RESELLER_EXEMPTION"),
            TaxExemption::US_MA_RESELLER_EXEMPTION => String::from("US_MA_RESELLER_EXEMPTION"),
            TaxExemption::US_MD_RESELLER_EXEMPTION => String::from("US_MD_RESELLER_EXEMPTION"),
            TaxExemption::US_ME_RESELLER_EXEMPTION => String::from("US_ME_RESELLER_EXEMPTION"),
            TaxExemption::US_MI_RESELLER_EXEMPTION => String::from("US_MI_RESELLER_EXEMPTION"),
            TaxExemption::US_MN_RESELLER_EXEMPTION => String::from("US_MN_RESELLER_EXEMPTION"),
            TaxExemption::US_MO_RESELLER_EXEMPTION => String::from("US_MO_RESELLER_EXEMPTION"),
            TaxExemption::US_MS_RESELLER_EXEMPTION => String::from("US_MS_RESELLER_EXEMPTION"),
            TaxExemption::US_MT_RESELLER_EXEMPTION => String::from("US_MT_RESELLER_EXEMPTION"),
            TaxExemption::US_NC_RESELLER_EXEMPTION => String::from("US_NC_RESELLER_EXEMPTION"),
            TaxExemption::US_ND_RESELLER_EXEMPTION => String::from("US_ND_RESELLER_EXEMPTION"),
            TaxExemption::US_NE_RESELLER_EXEMPTION => String::from("US_NE_RESELLER_EXEMPTION"),
            TaxExemption::US_NH_RESELLER_EXEMPTION => String::from("US_NH_RESELLER_EXEMPTION"),
            TaxExemption::US_NJ_RESELLER_EXEMPTION => String::from("US_NJ_RESELLER_EXEMPTION"),
            TaxExemption::US_NM_RESELLER_EXEMPTION => String::from("US_NM_RESELLER_EXEMPTION"),
            TaxExemption::US_NV_RESELLER_EXEMPTION => String::from("US_NV_RESELLER_EXEMPTION"),
            TaxExemption::US_NY_RESELLER_EXEMPTION => String::from("US_NY_RESELLER_EXEMPTION"),
            TaxExemption::US_OH_RESELLER_EXEMPTION => String::from("US_OH_RESELLER_EXEMPTION"),
            TaxExemption::US_OK_RESELLER_EXEMPTION => String::from("US_OK_RESELLER_EXEMPTION"),
            TaxExemption::US_OR_RESELLER_EXEMPTION => String::from("US_OR_RESELLER_EXEMPTION"),
            TaxExemption::US_PA_RESELLER_EXEMPTION => String::from("US_PA_RESELLER_EXEMPTION"),
            TaxExemption::US_RI_RESELLER_EXEMPTION => String::from("US_RI_RESELLER_EXEMPTION"),
            TaxExemption::US_SC_RESELLER_EXEMPTION => String::from("US_SC_RESELLER_EXEMPTION"),
            TaxExemption::US_SD_RESELLER_EXEMPTION => String::from("US_SD_RESELLER_EXEMPTION"),
            TaxExemption::US_TN_RESELLER_EXEMPTION => String::from("US_TN_RESELLER_EXEMPTION"),
            TaxExemption::US_TX_RESELLER_EXEMPTION => String::from("US_TX_RESELLER_EXEMPTION"),
            TaxExemption::US_UT_RESELLER_EXEMPTION => String::from("US_UT_RESELLER_EXEMPTION"),
            TaxExemption::US_VA_RESELLER_EXEMPTION => String::from("US_VA_RESELLER_EXEMPTION"),
            TaxExemption::US_VT_RESELLER_EXEMPTION => String::from("US_VT_RESELLER_EXEMPTION"),
            TaxExemption::US_WA_RESELLER_EXEMPTION => String::from("US_WA_RESELLER_EXEMPTION"),
            TaxExemption::US_WI_RESELLER_EXEMPTION => String::from("US_WI_RESELLER_EXEMPTION"),
            TaxExemption::US_WV_RESELLER_EXEMPTION => String::from("US_WV_RESELLER_EXEMPTION"),
            TaxExemption::US_WY_RESELLER_EXEMPTION => String::from("US_WY_RESELLER_EXEMPTION"),
        }
    }
}

// Implement Display trait for easy printing
impl std::fmt::Display for TaxExemption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Serialize for TaxExemption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TaxExemption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaxExemptionVisitor;

        impl<'de> Visitor<'de> for TaxExemptionVisitor {
            type Value = TaxExemption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid customer sms marketing opt in level type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TaxExemption::from_str(value))
            }
        }

        deserializer.deserialize_str(TaxExemptionVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerSmsMarketingConsentInput {
    #[serde(rename="consentUpdatedAt")]
    pub consent_updated_at: Option<DateTime<Utc>>,
    #[serde(rename="marketingOptInLevel")]
    pub marketing_opt_in_level: Option<CustomerMarketingOptInLevel>,
    #[serde(rename="marketingState")]
    pub marketing_state: Option<CustomerSmsMarketingState>
}

#[derive(Debug, Clone)]
pub enum CustomerSmsMarketingState{
    NOTSUBSCRIBED,
    PENDING,
    REDACTED,
    SUBSCRIBED,
    UNSUBSCRIBED
}

impl CustomerSmsMarketingState {
    pub fn from_str(code: &str) -> CustomerSmsMarketingState {
        match code {
           "PENDING" => CustomerSmsMarketingState::PENDING,
           "REDACTED" => CustomerSmsMarketingState::REDACTED,
           "SUBSCRIBED" => CustomerSmsMarketingState::SUBSCRIBED,
           "UNSUBSCRIBED" => CustomerSmsMarketingState::UNSUBSCRIBED,
           "NOT_SUBSCRIBED" | _=> CustomerSmsMarketingState::NOTSUBSCRIBED,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CustomerSmsMarketingState::REDACTED => String::from("SINGLE_OPT_IN"),
            CustomerSmsMarketingState::SUBSCRIBED => String::from("UNKNOWN"),
            CustomerSmsMarketingState::UNSUBSCRIBED => String::from("UNSUBSCRIBED"),
            CustomerSmsMarketingState::PENDING => String::from("PENDING"),
            CustomerSmsMarketingState::NOTSUBSCRIBED => String::from("NOT_SUBSCRIBED")
        }
    }
}

// Implement Display trait for easy printing
impl std::fmt::Display for CustomerSmsMarketingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Serialize for CustomerSmsMarketingState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CustomerSmsMarketingState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomerSmsMarketingStateVisitor;

        impl<'de> Visitor<'de> for CustomerSmsMarketingStateVisitor {
            type Value = CustomerSmsMarketingState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid customer sms marketing opt in level type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CustomerSmsMarketingState::from_str(value))
            }
        }

        deserializer.deserialize_str(CustomerSmsMarketingStateVisitor)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerEmailMarketingConsentInput {
    #[serde(rename="consentUpdatedAt")]
    pub consent_updated_at: Option<DateTime<Utc>>,
    #[serde(rename="marketingOptInLevel")]
    pub marketing_opt_in_level: Option<CustomerMarketingOptInLevel>,
    #[serde(rename="marketingState")]
    pub marketing_state: Option<CustomerEmailMarketingState>
}

#[derive(Debug, Clone)]
pub enum CustomerEmailMarketingState {
    INVALID,
    NOTSUBSCRIBED,
    PENDING,
    REDACTED,
    SUBSCRIBED,
    UNSUBSCRIBED,
}

impl CustomerEmailMarketingState {
    pub fn from_str(code: &str) -> CustomerEmailMarketingState {
        match code {
           "INVALID" => CustomerEmailMarketingState::INVALID,
           "REDACTED" => CustomerEmailMarketingState::REDACTED,
           "SUBSCRIBED" => CustomerEmailMarketingState::SUBSCRIBED,
           "UNSUBSCRIBED" => CustomerEmailMarketingState::UNSUBSCRIBED,
           "PENDING" => CustomerEmailMarketingState::PENDING,
           "NOT_SUBSCRIBED" | _=> CustomerEmailMarketingState::NOTSUBSCRIBED,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CustomerEmailMarketingState::INVALID => String::from("CONFIRMED_OPT_IN"),
            CustomerEmailMarketingState::REDACTED => String::from("SINGLE_OPT_IN"),
            CustomerEmailMarketingState::SUBSCRIBED => String::from("UNKNOWN"),
            CustomerEmailMarketingState::UNSUBSCRIBED => String::from("UNSUBSCRIBED"),
            CustomerEmailMarketingState::PENDING => String::from("PENDING"),
            CustomerEmailMarketingState::NOTSUBSCRIBED => String::from("NOT_SUBSCRIBED")
        }
    }
}

// Implement Display trait for easy printing
impl std::fmt::Display for CustomerEmailMarketingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Serialize for CustomerEmailMarketingState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CustomerEmailMarketingState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomerEmailMarketingStateVisitor;

        impl<'de> Visitor<'de> for CustomerEmailMarketingStateVisitor {
            type Value = CustomerEmailMarketingState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid customer marketing opt in level type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CustomerEmailMarketingState::from_str(value))
            }
        }

        deserializer.deserialize_str(CustomerEmailMarketingStateVisitor)
    }
}


#[derive(Debug, Clone)]
pub enum CustomerMarketingOptInLevel {
    CONFIRMEDOPTIN,
    SINGLEOPTIN,
    UNKNOWN
}

impl CustomerMarketingOptInLevel {
    pub fn from_str(code: &str) -> CustomerMarketingOptInLevel {
        match code {
           "CONFIRMED_OPT_IN" => CustomerMarketingOptInLevel::CONFIRMEDOPTIN,
           "SINGLE_OPT_IN" => CustomerMarketingOptInLevel::SINGLEOPTIN,
           "UNKNOWN" | _=> CustomerMarketingOptInLevel::UNKNOWN
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CustomerMarketingOptInLevel::CONFIRMEDOPTIN => String::from("CONFIRMED_OPT_IN"),
            CustomerMarketingOptInLevel::SINGLEOPTIN => String::from("SINGLE_OPT_IN"),
            CustomerMarketingOptInLevel::UNKNOWN => String::from("UNKNOWN")
        }
    }
}

// Implement Display trait for easy printing
impl std::fmt::Display for CustomerMarketingOptInLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Serialize for CustomerMarketingOptInLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CustomerMarketingOptInLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomerMarketingOptInLevelVisitor;

        impl<'de> Visitor<'de> for CustomerMarketingOptInLevelVisitor {
            type Value = CustomerMarketingOptInLevel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid customer marketing opt in level type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CustomerMarketingOptInLevel::from_str(value))
            }
        }

        deserializer.deserialize_str(CustomerMarketingOptInLevelVisitor)
    }
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MailingAddressInput {
    #[serde(rename="address1")]
    pub address1: Option<String>,
    #[serde(rename="address2")]
    pub address2: Option<String>,
    pub city: Option<String>,
    pub company: Option<String>,
    #[serde(rename="countryCode")]
    pub country_code: Option<CountryCode>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub phone: Option<String>,          // Ideally formatted as E.164
    #[serde(rename="provinceCode")]
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CountryCode {
    AC, AD, AE, AF, AG, AI, AL, AM, AN, AO, AR, AT, AU, AW, AX, AZ,
    BA, BB, BD, BE, BF, BG, BH, BI, BJ, BL, BM, BN, BO, BQ, BR, BS, BT, BV, BW, BY, BZ,
    CA, CC, CD, CF, CG, CH, CI, CK, CL, CM, CN, CO, CR, CU, CV, CW, CX, CY, CZ,
    DE, DJ, DK, DM, DO, DZ,
    EC, EE, EG, EH, ER, ES, ET,
    FI, FJ, FK, FO, FR,
    GA, GB, GD, GE, GF, GG, GH, GI, GL, GM, GN, GP, GQ, GR, GS, GT, GW, GY,
    HK, HM, HN, HR, HT, HU,
    ID, IE, IL, IM, IN, IO, IQ, IR, IS, IT,
    JE, JM, JO, JP,
    KE, KG, KH, KI, KM, KN, KP, KR, KW, KY, KZ,
    LA, LB, LC, LI, LK, LR, LS, LT, LU, LV, LY,
    MA, MC, MD, ME, MF, MG, MK, ML, MM, MN, MO, MQ, MR, MS, MT, MU, MV, MW, MX, MY, MZ,
    NA, NC, NE, NF, NG, NI, NL, NO, NP, NR, NU, NZ,
    OM,
    PA, PE, PF, PG, PH, PK, PL, PM, PN, PS, PT, PY,
    QA,
    RE, RO, RS, RU, RW,
    SA, SB, SC, SD, SE, SG, SH, SI, SJ, SK, SL, SM, SN, SO, SR, SS, ST, SV, SX, SY, SZ,
    TA, TC, TD, TF, TG, TH, TJ, TK, TL, TM, TN, TO, TR, TT, TV, TW, TZ,
    UA, UG, UM, US, UY, UZ,
    VA, VC, VE, VG, VN, VU,
    WF, WS,
    XK,
    YE, YT,
    ZA, ZM, ZW, ZZ, NotImplemented(String)
}

impl CountryCode {
    pub fn from_str(code: &str) -> Result<CountryCode, String> {
        match code.to_uppercase().as_str() {
            "US" | "UnitedStates" => Ok(CountryCode::US),
            "CA" => Ok(CountryCode::CA),
            "GB" => Ok(CountryCode::GB),
            // Add other implemented country codes here
            _ => Ok(CountryCode::NotImplemented(code.to_string())),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CountryCode::US => String::from("US"),
            CountryCode::CA => String::from("CA"),
            CountryCode::GB => String::from("GB"),
            // Add other implemented country codes here
            CountryCode::NotImplemented(code) => code.clone(),
            _=> todo!()  // Since we can write out the others
        }
    }
}

// Implement Display trait for easy printing
impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}