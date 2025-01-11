#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a TryFrom or FromStr implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "Accounts"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Accounts\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"data\","]
#[doc = "    \"filter\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"data\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/definitions/Datum\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"filter\": {"]
#[doc = "      \"$ref\": \"#/definitions/Filter\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Accounts {
    pub data: ::std::vec::Vec<Datum>,
    pub filter: Filter,
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
}
impl ::std::convert::From<&Accounts> for Accounts {
    fn from(value: &Accounts) -> Self {
        value.clone()
    }
}
impl Accounts {
    pub fn builder() -> builder::Accounts {
        Default::default()
    }
}
#[doc = "Datum"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Datum\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"network\","]
#[doc = "    \"ownerAddress\","]
#[doc = "    \"recipientAddress\","]
#[doc = "    \"version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"network\": {"]
#[doc = "      \"$ref\": \"#/definitions/Network\""]
#[doc = "    },"]
#[doc = "    \"ownerAddress\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"recipientAddress\": {"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"version\": {"]
#[doc = "      \"$ref\": \"#/definitions/Version\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Datum {
    pub id: i64,
    pub network: Network,
    #[serde(rename = "ownerAddress")]
    pub owner_address: ::std::string::String,
    #[serde(rename = "recipientAddress")]
    pub recipient_address: ::std::option::Option<::std::string::String>,
    pub version: Version,
}
impl ::std::convert::From<&Datum> for Datum {
    fn from(value: &Datum) -> Self {
        value.clone()
    }
}
impl Datum {
    pub fn builder() -> builder::Datum {
        Default::default()
    }
}
#[doc = "Filter"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Filter\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"page\","]
#[doc = "    \"perPage\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"page\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"perPage\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    pub page: i64,
    #[serde(rename = "perPage")]
    pub per_page: i64,
}
impl ::std::convert::From<&Filter> for Filter {
    fn from(value: &Filter) -> Self {
        value.clone()
    }
}
impl Filter {
    pub fn builder() -> builder::Filter {
        Default::default()
    }
}
#[doc = "Network"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Network\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"mainnet\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
}
impl ::std::convert::From<&Self> for Network {
    fn from(value: &Network) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Network {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}
impl ::std::str::FromStr for Network {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "mainnet" => Ok(Self::Mainnet),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Network {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Network {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Network {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Version"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Version\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"v4\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Version {
    #[serde(rename = "v4")]
    V4,
}
impl ::std::convert::From<&Self> for Version {
    fn from(value: &Version) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Version {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::V4 => write!(f, "v4"),
        }
    }
}
impl ::std::str::FromStr for Version {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "v4" => Ok(Self::V4),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Version {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Version {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Version {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Accounts {
        data: ::std::result::Result<::std::vec::Vec<super::Datum>, ::std::string::String>,
        filter: ::std::result::Result<super::Filter, ::std::string::String>,
        type_: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for Accounts {
        fn default() -> Self {
            Self {
                data: Err("no value supplied for data".to_string()),
                filter: Err("no value supplied for filter".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl Accounts {
        pub fn data<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Datum>>,
            T::Error: ::std::fmt::Display,
        {
            self.data = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for data: {}", e));
            self
        }
        pub fn filter<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Filter>,
            T::Error: ::std::fmt::Display,
        {
            self.filter = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for filter: {}", e));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Accounts> for super::Accounts {
        type Error = super::error::ConversionError;
        fn try_from(value: Accounts) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                data: value.data?,
                filter: value.filter?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::Accounts> for Accounts {
        fn from(value: super::Accounts) -> Self {
            Self {
                data: Ok(value.data),
                filter: Ok(value.filter),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Datum {
        id: ::std::result::Result<i64, ::std::string::String>,
        network: ::std::result::Result<super::Network, ::std::string::String>,
        owner_address: ::std::result::Result<::std::string::String, ::std::string::String>,
        recipient_address: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        version: ::std::result::Result<super::Version, ::std::string::String>,
    }
    impl ::std::default::Default for Datum {
        fn default() -> Self {
            Self {
                id: Err("no value supplied for id".to_string()),
                network: Err("no value supplied for network".to_string()),
                owner_address: Err("no value supplied for owner_address".to_string()),
                recipient_address: Err("no value supplied for recipient_address".to_string()),
                version: Err("no value supplied for version".to_string()),
            }
        }
    }
    impl Datum {
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {}", e));
            self
        }
        pub fn network<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Network>,
            T::Error: ::std::fmt::Display,
        {
            self.network = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for network: {}", e));
            self
        }
        pub fn owner_address<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.owner_address = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for owner_address: {}", e));
            self
        }
        pub fn recipient_address<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.recipient_address = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for recipient_address: {}",
                    e
                )
            });
            self
        }
        pub fn version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Version>,
            T::Error: ::std::fmt::Display,
        {
            self.version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for version: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Datum> for super::Datum {
        type Error = super::error::ConversionError;
        fn try_from(value: Datum) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                id: value.id?,
                network: value.network?,
                owner_address: value.owner_address?,
                recipient_address: value.recipient_address?,
                version: value.version?,
            })
        }
    }
    impl ::std::convert::From<super::Datum> for Datum {
        fn from(value: super::Datum) -> Self {
            Self {
                id: Ok(value.id),
                network: Ok(value.network),
                owner_address: Ok(value.owner_address),
                recipient_address: Ok(value.recipient_address),
                version: Ok(value.version),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Filter {
        page: ::std::result::Result<i64, ::std::string::String>,
        per_page: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for Filter {
        fn default() -> Self {
            Self {
                page: Err("no value supplied for page".to_string()),
                per_page: Err("no value supplied for per_page".to_string()),
            }
        }
    }
    impl Filter {
        pub fn page<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.page = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for page: {}", e));
            self
        }
        pub fn per_page<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.per_page = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for per_page: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Filter> for super::Filter {
        type Error = super::error::ConversionError;
        fn try_from(value: Filter) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                page: value.page?,
                per_page: value.per_page?,
            })
        }
    }
    impl ::std::convert::From<super::Filter> for Filter {
        fn from(value: super::Filter) -> Self {
            Self {
                page: Ok(value.page),
                per_page: Ok(value.per_page),
            }
        }
    }
}
