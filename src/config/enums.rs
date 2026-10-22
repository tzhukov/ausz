use std::fmt;

// ---------------------------------------------------------------------------
// SupportedScope
// ---------------------------------------------------------------------------

/// OIDC scopes that this provider supports.
/// These are the standard scopes from the OIDC spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedScope {
    OpenId,
    Profile,
    Email,
    EmailVerified,
    Name,
    PreferredUsername,
    Nickname,
    Address,
    Phone,
    OfflineAccess,
}

impl SupportedScope {
    /// The string representation used in OIDC requests and responses.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenId => "openid",
            Self::Profile => "profile",
            Self::Email => "email",
            Self::EmailVerified => "email_verified",
            Self::Name => "name",
            Self::PreferredUsername => "preferred_username",
            Self::Nickname => "nickname",
            Self::Address => "address",
            Self::Phone => "phone",
            Self::OfflineAccess => "offline_access",
        }
    }

    /// Parse from a TOML config string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openid" => Some(Self::OpenId),
            "profile" => Some(Self::Profile),
            "email" => Some(Self::Email),
            "email_verified" => Some(Self::EmailVerified),
            "name" => Some(Self::Name),
            "preferred_username" => Some(Self::PreferredUsername),
            "nickname" => Some(Self::Nickname),
            "address" => Some(Self::Address),
            "phone" => Some(Self::Phone),
            "offline_access" => Some(Self::OfflineAccess),
            _ => None,
        }
    }
}

impl fmt::Display for SupportedScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// SupportedResponseType
// ---------------------------------------------------------------------------

/// OIDC response types that this provider supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedResponseType {
    Code,
    CodeIdToken,
    IdTokenToken,
    CodeToken,
}

impl SupportedResponseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::CodeIdToken => "code id_token",
            Self::IdTokenToken => "id_token token",
            Self::CodeToken => "code token",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code" => Some(Self::Code),
            "code id_token" => Some(Self::CodeIdToken),
            "id_token token" => Some(Self::IdTokenToken),
            "code token" => Some(Self::CodeToken),
            _ => None,
        }
    }
}

impl fmt::Display for SupportedResponseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// SupportedGrantType
// ---------------------------------------------------------------------------

/// OIDC grant types that this provider supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedGrantType {
    AuthorizationCode,
    RefreshToken,
    ClientCredentials,
    DeviceCode,
}

impl SupportedGrantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorizationCode => "authorization_code",
            Self::RefreshToken => "refresh_token",
            Self::ClientCredentials => "client_credentials",
            Self::DeviceCode => "urn:ietf:params:oauth:grant-type:device_code",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "authorization_code" => Some(Self::AuthorizationCode),
            "refresh_token" => Some(Self::RefreshToken),
            "client_credentials" => Some(Self::ClientCredentials),
            "urn:ietf:params:oauth:grant-type:device_code" => Some(Self::DeviceCode),
            _ => None,
        }
    }
}

impl fmt::Display for SupportedGrantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// SupportedSubjectType
// ---------------------------------------------------------------------------

/// OIDC subject types that this provider supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedSubjectType {
    Public,
}

impl SupportedSubjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "public" => Some(Self::Public),
            _ => None,
        }
    }
}

impl fmt::Display for SupportedSubjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// SigningAlgorithm
// ---------------------------------------------------------------------------

/// JWT signing algorithms supported by this provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigningAlgorithm {
    RS256,
    ES256,
}

impl SigningAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RS256 => "RS256",
            Self::ES256 => "ES256",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "RS256" => Some(Self::RS256),
            "ES256" => Some(Self::ES256),
            _ => None,
        }
    }
}

impl fmt::Display for SigningAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// CodeChallengeMethod
// ---------------------------------------------------------------------------

/// PKCE code challenge methods supported by this provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeChallengeMethod {
    S256,
    Plain,
}

impl CodeChallengeMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S256 => "S256",
            Self::Plain => "plain",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "S256" => Some(Self::S256),
            "plain" => Some(Self::Plain),
            _ => None,
        }
    }
}

impl fmt::Display for CodeChallengeMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
