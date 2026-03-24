use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DiscountType {
    AutomaticDiscount,
    CodeDiscount,
    Manual,
}

impl DiscountType {
    pub fn from_str(discount_type: &str) -> DiscountType {
        match discount_type {
            "AUTOMATIC_DISCOUNT" => DiscountType::AutomaticDiscount,
            "CODE_DISCOUNT" => DiscountType::CodeDiscount,
            "MANUAL" | _ => DiscountType::Manual,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DiscountType::AutomaticDiscount => String::from("AUTOMATIC_DISCOUNT"),
            DiscountType::CodeDiscount => String::from("CODE_DISCOUNT"),
            DiscountType::Manual => String::from("MANUAL"),
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DiscountTargetType {
    LineItem,
    ShippingLine,
}

impl DiscountTargetType {
    pub fn from_str(target_type: &str) -> DiscountTargetType {
        match target_type {
            "LINE_ITEM" => DiscountTargetType::LineItem,
            "SHIPPING_LINE" => DiscountTargetType::ShippingLine,
            _ => DiscountTargetType::LineItem, // Default case
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DiscountTargetType::LineItem => String::from("LINE_ITEM"),
            DiscountTargetType::ShippingLine => String::from("SHIPPING_LINE"),
        }
    }
}