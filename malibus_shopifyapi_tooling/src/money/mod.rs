use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::str::FromStr;
use std::ops::{ Add, Sub, Mul, Div };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoneyBag {
    #[serde(rename="presentmentMoney")]
    pub presentment_money: Option<MoneyV2>,
    #[serde(rename="shopMoney")]
    pub shop_money: Option<MoneyV2>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoneyV2{
    pub amount: Option<Decimal>,
    #[serde(rename="currencyCode")]
    pub currency_code: Option<CurrencyCode>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoneyInput {
    pub amount: Option<String>,                   // required when applicable (didn't write custom de/se for Decimal scalar - stick with string)
    #[serde(rename="currencyCode")]
    pub currency_code: Option<CurrencyCode>       // required when applicable
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CurrencyCode {
    AED,
    //United Arab Emirates Dirham (AED).
    AFN,
    //Afghan Afghani (AFN).
    ALL,
    //Albanian Lek (ALL).
    AMD,
    //Armenian Dram (AMD).
    ANG,
    //Netherlands Antillean Guilder.
    AOA,
    //Angolan Kwanza (AOA).
    ARS,
    //Argentine Pesos (ARS).
    AUD,
    //Australian Dollars (AUD).
    AWG,
    //Aruban Florin (AWG).
    AZN,
    //Azerbaijani Manat (AZN).
    BAM,
    //Bosnia and Herzegovina Convertible Mark (BAM).
    BBD,
    //Barbadian Dollar (BBD).
    BDT,
    //Bangladesh Taka (BDT).
    BGN,
    //Bulgarian Lev (BGN).
    BHD,
    //Bahraini Dinar (BHD).
    BIF,
    //Burundian Franc (BIF).
    BMD,
    //Bermudian Dollar (BMD).
    BND,
    //Brunei Dollar (BND).
    BOB,
    //Bolivian Boliviano (BOB).
    BRL,
    //Brazilian Real (BRL).
    BSD,
    //Bahamian Dollar (BSD).
    BTN,
    //Bhutanese Ngultrum (BTN).
    BWP,
    //Botswana Pula (BWP).
    BYN,
    //Belarusian Ruble (BYN).
    BZD,
    //Belize Dollar (BZD).
    CAD,
    //Canadian Dollars (CAD).
    CDF,
    //Congolese franc (CDF).
    CHF,
    //Swiss Francs (CHF).
    CLP,
    //Chilean Peso (CLP).
    CNY,
    //Chinese Yuan Renminbi (CNY).
    COP,
    //Colombian Peso (COP).
    CRC,
    //Costa Rican Colones (CRC).
    CVE,
    //Cape Verdean escudo (CVE).
    CZK,
    //Czech Koruny (CZK).
    DJF,
    //Djiboutian Franc (DJF).
    DKK,
    //Danish Kroner (DKK).
    DOP,
    //Dominican Peso (DOP).
    DZD,
    //Algerian Dinar (DZD).
    EGP,
    //Egyptian Pound (EGP).
    ERN,
    //Eritrean Nakfa (ERN).
    ETB,
    //Ethiopian Birr (ETB).
    EUR,
    //Euro (EUR).
    FJD,
    //Fijian Dollars (FJD).
    FKP,
    //Falkland Islands Pounds (FKP).
    GBP,
    //United Kingdom Pounds (GBP).
    GEL,
    //Georgian Lari (GEL).
    GHS,
    //Ghanaian Cedi (GHS).
    GIP,
    //Gibraltar Pounds (GIP).
    GMD,
    //Gambian Dalasi (GMD).
    GNF,
    //Guinean Franc (GNF).
    GTQ,
    //Guatemalan Quetzal (GTQ).
    GYD,
    //Guyanese Dollar (GYD).
    HKD,
    //Hong Kong Dollars (HKD).
    HNL,
    //Honduran Lempira (HNL).
    HRK,
    //Croatian Kuna (HRK).
    HTG,
    //Haitian Gourde (HTG).
    HUF,
    //Hungarian Forint (HUF).
    IDR,
    //Indonesian Rupiah (IDR).
    ILS,
    //Israeli New Shekel (NIS).
    INR,
    //Indian Rupees (INR).
    IQD,
    //Iraqi Dinar (IQD).
    IRR,
    //Iranian Rial (IRR).
    ISK,
    //Icelandic Kronur (ISK).
    JEP,
    //Jersey Pound.
    JMD,
    //Jamaican Dollars (JMD).
    JOD,
    //Jordanian Dinar (JOD).
    JPY,
    //Japanese Yen (JPY).
    KES,
    //Kenyan Shilling (KES).
    KGS,
    //Kyrgyzstani Som (KGS).
    KHR,
    //Cambodian Riel.
    KID,
    //Kiribati Dollar (KID).
    KMF,
    //Comorian Franc (KMF).
    KRW,
    //South Korean Won (KRW).
    KWD,
    //Kuwaiti Dinar (KWD).
    KYD,
    //Cayman Dollars (KYD).
    KZT,
    //Kazakhstani Tenge (KZT).
    LAK,
    //Laotian Kip (LAK).
    LBP,
    //Lebanese Pounds (LBP).
    LKR,
    //Sri Lankan Rupees (LKR).
    LRD,
    //Liberian Dollar (LRD).
    LSL,
    //Lesotho Loti (LSL).
    LTL,
    //Lithuanian Litai (LTL).
    LVL,
    //Latvian Lati (LVL).
    LYD,
    //Libyan Dinar (LYD).
    MAD,
    //Moroccan Dirham.
    MDL,
    //Moldovan Leu (MDL).
    MGA,
    //Malagasy Ariary (MGA).
    MKD,
    //Macedonia Denar (MKD).
    MMK,
    //Burmese Kyat (MMK).
    MNT,
    //Mongolian Tugrik.
    MOP,
    //Macanese Pataca (MOP).
    MRU,
    //Mauritanian Ouguiya (MRU).
    MUR,
    //Mauritian Rupee (MUR).
    MVR,
    //Maldivian Rufiyaa (MVR).
    MWK,
    //Malawian Kwacha (MWK).
    MXN,
    //Mexican Pesos (MXN).
    MYR,
    //Malaysian Ringgits (MYR).
    MZN,
    //Mozambican Metical.
    NAD,
    //Namibian Dollar.
    NGN,
    //Nigerian Naira (NGN).
    NIO,
    //Nicaraguan Córdoba (NIO).
    NOK,
    //Norwegian Kroner (NOK).
    NPR,
    //Nepalese Rupee (NPR).
    NZD,
    //New Zealand Dollars (NZD).
    OMR,
    //Omani Rial (OMR).
    PAB,
    //Panamian Balboa (PAB).
    PEN,
    //Peruvian Nuevo Sol (PEN).
    PGK,
    //Papua New Guinean Kina (PGK).
    PHP,
    //Philippine Peso (PHP).
    PKR,
    //Pakistani Rupee (PKR).
    PLN,
    //Polish Zlotych (PLN).
    PYG,
    //Paraguayan Guarani (PYG).
    QAR,
    //Qatari Rial (QAR).
    RON,
    //Romanian Lei (RON).
    RSD,
    //Serbian dinar (RSD).
    RUB,
    //Russian Rubles (RUB).
    RWF,
    //Rwandan Franc (RWF).
    SAR,
    //Saudi Riyal (SAR).
    SBD,
    //Solomon Islands Dollar (SBD).
    SCR,
    //Seychellois Rupee (SCR).
    SDG,
    //Sudanese Pound (SDG).
    SEK,
    //Swedish Kronor (SEK).
    SGD,
    //Singapore Dollars (SGD).
    SHP,
    //Saint Helena Pounds (SHP).
    SLL,
    //Sierra Leonean Leone (SLL).
    SOS,
    //Somali Shilling (SOS).
    SRD,
    //Surinamese Dollar (SRD).
    SSP,
    //South Sudanese Pound (SSP).
    STN,
    //Sao Tome And Principe Dobra (STN).
    SYP,
    //Syrian Pound (SYP).
    SZL,
    //Swazi Lilangeni (SZL).
    THB,
    //Thai baht (THB).
    TJS,
    //Tajikistani Somoni (TJS).
    TMT,
    //Turkmenistani Manat (TMT).
    TND,
    //Tunisian Dinar (TND).
    TOP,
    //Tongan Pa'anga (TOP).
    TRY,
    //Turkish Lira (TRY).
    TTD,
    //Trinidad and Tobago Dollars (TTD).
    TWD,
    //Taiwan Dollars (TWD).
    TZS,
    //Tanzanian Shilling (TZS).
    UAH,
    //Ukrainian Hryvnia (UAH).
    UGX,
    //Ugandan Shilling (UGX).
    USD,
    //United States Dollars (USD).
    UYU,
    //Uruguayan Pesos (UYU).
    UZS,
    //Uzbekistan som (UZS).
    VED,
    //Venezuelan Bolivares (VED).
    VES,
    //Venezuelan Bolivares Soberanos (VES).
    VND,
    //Vietnamese đồng (VND).
    VUV,
    //Vanuatu Vatu (VUV).
    WST,
    //Samoan Tala (WST).
    XAF,
    //Central African CFA Franc (XAF).
    XCD,
    //East Caribbean Dollar (XCD).
    XOF,
    //West African CFA franc (XOF).
    XPF,
    //CFP Franc (XPF).
    XXX,
    //Unrecognized currency.
    YER,
    //Yemeni Rial (YER).
    ZAR,
    //South African Rand (ZAR).
    ZMW,
    //Zambian Kwacha (ZMW).
    BYR,
    //Belarusian Ruble (BYR).
    STD,
    //Sao Tome And Principe Dobra (STD).
    VEF,
    //Venezuelan Bolivares (VEF).
}

impl CurrencyCode {
    pub fn from_str(code: &str) ->  CurrencyCode {
        match code {
            "AED" =>  CurrencyCode::AED, 
            // Add others when needed!
            "CAD" => CurrencyCode::CAD,
            // ...
            "USD" | _=> CurrencyCode::USD,
        }
    }
    
    pub fn to_string(&self) -> String {
        match *self {
            CurrencyCode::AED => String::from("AED"),    
            CurrencyCode::USD => String::from("USD"),
            CurrencyCode::CAD => String::from("CAD"),
            // .. Add others when needed!
            _ => todo!(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money(String);

impl Money {
    // Create a new Money instance
    pub fn new(value: &str) -> Result<Self, &'static str> {
        // Validate the input value
        if Self::is_valid(value) {
            Ok(Money(value.to_string()))
        } else {
            Err("Invalid monetary value")
        }
    }

    // Validate the monetary value format
    fn is_valid(value: &str) -> bool {
        // Check if the value is a valid decimal number
        value.parse::<f64>().is_ok()
    }

    // Convert cents to a Money instance with dollar representation
    pub fn from_cents(cents_str: &str) -> Result<Self, &'static str> {
        let cents: i32 = cents_str.parse().map_err(|_| "Failed to parse string to integer")?;
        let dollars = cents as f64 / 100.0;
        let dollars_str = format!("{:.2}", dollars);
        Money::new(&dollars_str)
    }

    // Get the inner string representation
    pub fn value(&self) -> &str {
        &self.0
    }
}

// Implement custom serialization for Money
impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

// Implement custom deserialization for Money
impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MoneyVisitor;

        impl<'de> Visitor<'de> for MoneyVisitor {
            type Value = Money;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a monetary value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Money, E>
            where
                E: de::Error,
            {
                Money::new(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(MoneyVisitor)
    }
}


impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let sum = self.0.parse::<f64>().unwrap() + other.0.parse::<f64>().unwrap();
        Money(format!("{:.2}", sum))
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let difference = self.0.parse::<f64>().unwrap() - other.0.parse::<f64>().unwrap();
        Money(format!("{:.2}", difference))
    }
}

impl Mul<f64> for Money {
    type Output = Self;

    fn mul(self, factor: f64) -> Self {
        let product = self.0.parse::<f64>().unwrap() * factor;
        Money(format!("{:.2}", product))
    }
}

impl Div<f64> for Money {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        let quotient = self.0.parse::<f64>().unwrap() / divisor;
        Money(format!("{:.2}", quotient))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Decimal {
    value: String,
}

impl Decimal {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Decimal {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    // Optional: Parse to f64 when needed
    pub fn to_f64(&self) -> Option<f64> {
        self.value.parse().ok()
    }

    pub fn from_f64(value: f64) -> Self {
        Decimal {
            value: value.to_string(),
        }
    }
}

// Display implementation for easy printing
impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// FromStr implementation for string parsing
impl FromStr for Decimal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Basic validation that it's a valid decimal number
        if s.parse::<f64>().is_ok() {
            Ok(Decimal::new(s.to_string()))
        } else {
            Err("Invalid decimal format".to_string())
        }
    }
}

// Custom serialization to ensure it's always serialized as a string
impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

// Custom deserialization to handle both string and number inputs
impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        
        // First try to deserialize as string
        let value = String::deserialize(deserializer)?;
        
        // Validate that it's a valid decimal number
        if value.parse::<f64>().is_ok() {
            Ok(Decimal::new(value))
        } else {
            Err(D::Error::custom("Invalid decimal format"))
        }
    }
}