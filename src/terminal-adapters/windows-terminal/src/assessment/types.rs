use crate::Guid;
use serde_json::Value;

/// Caller-supplied bytes only; this type asserts neither inventory completeness nor freshness.
pub struct SuppliedSnapshot<'a> {
    pub settings_jsonc: &'a str,
    pub terminalhud_fragments: &'a [SuppliedFragment<'a>],
}

/// A document in the TerminalHUD fragment namespace, identified by the caller.
/// Identifiers are labels, never paths opened by this API.
pub struct SuppliedFragment<'a> {
    pub namespace: &'a str,
    pub document_id: &'a str,
    pub jsonc: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    UserProfile,
    UserDefaults,
    Fragment {
        namespace: String,
        document_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    Commandline,
    StartingDirectory,
    Environment,
    Elevate,
    ConnectionType,
    Hidden,
    Opacity,
    UseAcrylic,
    BackgroundImage,
    BackgroundImageOpacity,
    ColorScheme,
    UnfocusedOpacity,
    UnfocusedUseAcrylic,
    UnfocusedBackgroundImage,
    UnfocusedBackgroundImageOpacity,
    UnfocusedColorScheme,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedValue {
    /// No supplied value; inbox settings, built-in defaults and host state are not modeled.
    MissingBaseLayer,
    /// Unsupplied inbox profile defaults precede fragment parents in the real loader.
    InboxDefaultsUnresolved,
    /// Valid or potentially valid Terminal forms outside this evaluator's bounded subset.
    UnsupportedForm,
    /// A dynamic, disabled, missing or otherwise unmodeled source could change inheritance.
    UnresolvedSource,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyExpectation {
    /// Raw settings value, not an evaluated command, path, environment or color resource.
    Supplied {
        raw: Value,
        provenance: Provenance,
    },
    /// A fragment value that unsupplied inbox defaults could shadow. Never a resolved expectation.
    Candidate {
        raw: Value,
        provenance: Provenance,
        unresolved: UnresolvedValue,
    },
    Unresolved(UnresolvedValue),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedProperty {
    pub property: Property,
    pub expectation: PropertyExpectation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedProfile {
    pub guid: Guid,
    pub properties: Vec<ExpectedProperty>,
}

/// Possible branches if the runtime cannot find the requested GUID, even if present on disk.
/// Neither variant resolves a fallback profile or establishes safety.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FallbackAssessment {
    OrdinaryDefaultPossible { configured_guid: Option<Guid> },
    CommandMatchOrProfileDefaultsUnresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalReason {
    MalformedDocument,
    DuplicateJsonKey,
    InvalidContainer,
    InvalidField,
    MissingGuid,
    DuplicateGuid,
    ConflictingFragmentGuid,
    DuplicateFragmentInput,
    UnsupportedNamespace,
    UnsupportedUpdate,
    UnsupportedForm,
    UnresolvedSource,
    DisabledSource,
    MissingRequestedGuid,
    ElevatedProfile,
    NonzeroConnectionType,
    MissingProperty,
    InboxDefaultsUnresolved,
    FallbackUnproved,
    InventoryCompletenessUnproved,
    InputFreshnessUnproved,
    RuntimeRecognitionUnproved,
    LoadedCatalogUnproved,
    DedicatedProcessUnproved,
    LaunchEnvironmentUnproved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssessmentFinding {
    pub reason: RefusalReason,
    /// A supplied document/property label, not a discovered filesystem path.
    pub location: String,
}

/// Expected disk properties and refusals only. This is never a launch permit.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileAssessment {
    pub selected: Option<ExpectedProfile>,
    pub fallback: FallbackAssessment,
    /// Always contains unproved runtime/catalog/process/inventory/freshness findings.
    pub findings: Vec<AssessmentFinding>,
}

pub(super) fn finding(reason: RefusalReason, location: impl Into<String>) -> AssessmentFinding {
    AssessmentFinding {
        reason,
        location: location.into(),
    }
}
