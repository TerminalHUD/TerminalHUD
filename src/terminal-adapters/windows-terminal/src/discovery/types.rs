use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalChannel {
    Stable,
    Preview,
}
impl TerminalChannel {
    pub fn family(self) -> &'static str {
        match self {
            Self::Stable => "Microsoft.WindowsTerminal_8wekyb3d8bbwe",
            Self::Preview => "Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe",
        }
    }
    pub fn package_name(self) -> &'static str {
        match self {
            Self::Stable => "Microsoft.WindowsTerminal",
            Self::Preview => "Microsoft.WindowsTerminalPreview",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageArchitecture {
    X86,
    X64,
    Arm64,
    Other(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredPackageIdentity {
    pub name: String,
    pub publisher_id: String,
    pub resource_id: String,
    /// Package version, never an executable file or running Terminal version.
    pub package_version: [u16; 4],
    pub architecture: PackageArchitecture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryOperation {
    PackageFamily,
    PackageIdentity,
    PackagePath,
    LocalAppData,
    ExpectedApplicationPaths,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MalformedReply {
    Count,
    Size,
    Pointer,
    Terminator,
    Utf16,
    EmptyString,
    DuplicateName,
    IdentityMismatch,
    RelativePath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoveryFailure {
    /// Win32 code for package APIs; raw HRESULT bits for LocalAppData.
    Native {
        operation: DiscoveryOperation,
        code: u32,
    },
    Limit {
        operation: DiscoveryOperation,
    },
    ChangingResponse {
        operation: DiscoveryOperation,
    },
    Malformed {
        operation: DiscoveryOperation,
        reason: MalformedReply,
    },
    UnsupportedApplicationPackage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredPackageCandidate {
    pub channel: TerminalChannel,
    pub family: String,
    pub full_name: String,
    pub identity: Result<RegisteredPackageIdentity, DiscoveryFailure>,
    pub registered_install_path: Result<PathBuf, DiscoveryFailure>,
    /// Layout expectations only: no file existence, signature, hash or manifest validation.
    pub expected_executable: Result<PathBuf, DiscoveryFailure>,
    pub expected_settings: Result<PathBuf, DiscoveryFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredChannel {
    pub channel: TerminalChannel,
    /// Ok(empty) means the API reported no registrations in this family for this user.
    pub candidates: Result<Vec<RegisteredPackageCandidate>, DiscoveryFailure>,
}

/// A non-atomic observation of just two current-user registered package families.
/// No completeness, catalog recognition, executable authentication or launch authorization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredTerminalDiscovery {
    pub channels: [RegisteredChannel; 2],
    pub local_app_data: Result<PathBuf, DiscoveryFailure>,
}
