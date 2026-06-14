use core::fmt;

use aesynx_abi::ModelId;

use crate::{Confidence, PolicyError};

pub const MODEL_MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const MAX_MODEL_EVAL_STEPS: u32 = 1_000_000;
pub const MAX_MODEL_MEMORY_BYTES: u32 = 1_048_576;

/// A nonzero 256-bit hash metadata field.
///
/// # Security
///
/// Construction only proves the field is present and nonzero. It does not
/// verify that bytes loaded from storage match this hash.
#[derive(Clone, Copy, Eq)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    pub const fn new(bytes: [u8; 32]) -> Result<Self, PolicyError> {
        // SECURITY: this is a metadata-presence gate only. It is not
        // cryptographic hash verification.
        if all_zero_32(&bytes) {
            return Err(PolicyError::EmptyHash);
        }
        Ok(Self(bytes))
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Hash256(<redacted>)")
    }
}

impl PartialEq for Hash256 {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(&self.0, &other.0)
    }
}

/// A nonzero 64-byte signature metadata field.
///
/// # Security
///
/// Construction only proves the field is present and nonzero. It does not
/// cryptographically verify a manifest, model object, or model weights.
#[derive(Clone, Copy, Eq)]
pub struct Signature64([u8; 64]);

impl Signature64 {
    pub const fn new(bytes: [u8; 64]) -> Result<Self, PolicyError> {
        // SECURITY: this is a metadata-presence gate only. Real signature
        // validation must be added with the future model-loading backend.
        if all_zero_64(&bytes) {
            return Err(PolicyError::EmptySignature);
        }
        Ok(Self(bytes))
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; 64] {
        self.0
    }
}

impl fmt::Debug for Signature64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Signature64(<redacted>)")
    }
}

impl PartialEq for Signature64 {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelKind {
    FixedPointHeuristic,
    FixedPointTable,
    WasmComponent,
    NeuralNetwork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDomain {
    Scheduler,
    Admission,
    Capability,
    Security,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequiredTelemetryFields(u32);

impl RequiredTelemetryFields {
    pub const RUN_QUEUE_LEN: Self = Self(1 << 0);
    pub const IPC_DEPTH: Self = Self(1 << 1);
    pub const QUEUE_PRESSURE: Self = Self(1 << 2);
    pub const OBJECT_LOCALITY: Self = Self(1 << 3);
    pub const IDLE_RATIO: Self = Self(1 << 4);
    pub const MIGRATION_COST: Self = Self(1 << 5);
    pub const PRIORITY: Self = Self(1 << 6);

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn scheduler_minimum() -> Self {
        Self(
            Self::RUN_QUEUE_LEN.0 | Self::IPC_DEPTH.0 | Self::QUEUE_PRESSURE.0 | Self::IDLE_RATIO.0,
        )
    }

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, required: Self) -> bool {
        (self.0 & required.0) == required.0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelSafetyLimits {
    pub max_eval_steps: u32,
    pub max_memory_bytes: u32,
    pub max_confidence: Confidence,
    pub fallback_required: bool,
    pub required_telemetry: RequiredTelemetryFields,
}

impl ModelSafetyLimits {
    pub const fn new(
        max_eval_steps: u32,
        max_memory_bytes: u32,
        max_confidence: Confidence,
        fallback_required: bool,
        required_telemetry: RequiredTelemetryFields,
    ) -> Self {
        Self {
            max_eval_steps,
            max_memory_bytes,
            max_confidence,
            fallback_required,
            required_telemetry,
        }
    }

    pub const fn scheduler_default() -> Self {
        Self {
            max_eval_steps: 10_000,
            max_memory_bytes: 64 * 1024,
            max_confidence: Confidence::ZERO,
            fallback_required: true,
            required_telemetry: RequiredTelemetryFields::scheduler_minimum(),
        }
    }
}

/// Model metadata submitted to the policy layer.
///
/// A manifest is data, not executable code. Use
/// [`ModelObjectManifest::validate_for_domain`] before admitting it to a
/// policy domain.
///
/// `PartialEq` is for structural tests and non-authentication comparisons. It
/// is not a constant-time manifest verifier; future cryptographic checks must
/// verify signatures and hashes directly.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ModelObjectManifest {
    pub id: ModelId,
    pub schema_version: u16,
    pub kind: ModelKind,
    pub domain: PolicyDomain,
    pub input_schema_hash: Hash256,
    pub output_schema_hash: Hash256,
    pub weights_hash: Hash256,
    pub signature: Signature64,
    pub safety_limits: ModelSafetyLimits,
}

impl fmt::Debug for ModelObjectManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelObjectManifest")
            .field("id", &"<redacted>")
            .field("schema_version", &self.schema_version)
            .field("kind", &self.kind)
            .field("domain", &self.domain)
            .field("input_schema_hash", &self.input_schema_hash)
            .field("output_schema_hash", &self.output_schema_hash)
            .field("weights_hash", &self.weights_hash)
            .field("signature", &self.signature)
            .field("safety_limits", &self.safety_limits)
            .finish()
    }
}

impl ModelObjectManifest {
    pub const fn validate_for_domain(
        self,
        expected_domain: PolicyDomain,
    ) -> Result<ValidatedModelManifest, PolicyError> {
        if self.id.get() == 0 {
            return Err(PolicyError::EmptyModel);
        }
        if self.schema_version != MODEL_MANIFEST_SCHEMA_VERSION {
            return Err(PolicyError::UnsupportedSchema);
        }
        if !matches!(
            self.kind,
            ModelKind::FixedPointHeuristic | ModelKind::FixedPointTable
        ) {
            return Err(PolicyError::UnsupportedModelKind);
        }
        if !same_domain(self.domain, expected_domain) {
            return Err(PolicyError::UnsupportedDomain);
        }
        if !self.safety_limits.fallback_required {
            return Err(PolicyError::FallbackRequired);
        }
        if self.safety_limits.max_eval_steps == 0
            || self.safety_limits.max_eval_steps > MAX_MODEL_EVAL_STEPS
            || self.safety_limits.max_memory_bytes == 0
            || self.safety_limits.max_memory_bytes > MAX_MODEL_MEMORY_BYTES
        {
            return Err(PolicyError::ResourceLimitExceeded);
        }
        if same_domain(expected_domain, PolicyDomain::Scheduler)
            && !self
                .safety_limits
                .required_telemetry
                .contains(RequiredTelemetryFields::scheduler_minimum())
        {
            return Err(PolicyError::FeatureOutOfRange);
        }

        Ok(ValidatedModelManifest { manifest: self })
    }
}

/// A manifest that passed structural and policy validation.
///
/// Validation covers schema version, policy domain, supported model kind,
/// bounded resource limits, required fallback, and domain-specific telemetry
/// requirements.
///
/// # Security
///
/// This does not verify [`Hash256`] or [`Signature64`] cryptographically.
/// Callers that load model weights must perform real signature/hash
/// verification before trusting the metadata carried by this type.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ValidatedModelManifest {
    manifest: ModelObjectManifest,
}

impl fmt::Debug for ValidatedModelManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedModelManifest")
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl ValidatedModelManifest {
    #[must_use]
    pub const fn manifest(self) -> ModelObjectManifest {
        self.manifest
    }
}

const fn same_domain(left: PolicyDomain, right: PolicyDomain) -> bool {
    matches!(
        (left, right),
        (PolicyDomain::Scheduler, PolicyDomain::Scheduler)
            | (PolicyDomain::Admission, PolicyDomain::Admission)
            | (PolicyDomain::Capability, PolicyDomain::Capability)
            | (PolicyDomain::Security, PolicyDomain::Security)
    )
}

const fn all_zero_32(bytes: &[u8; 32]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0 {
            return false;
        }
        index += 1;
    }
    true
}

const fn all_zero_64(bytes: &[u8; 64]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0 {
            return false;
        }
        index += 1;
    }
    true
}

fn constant_time_eq<const LEN: usize>(left: &[u8; LEN], right: &[u8; LEN]) -> bool {
    let mut diff = 0u8;
    let mut index = 0;
    while index < LEN {
        diff |= left[index] ^ right[index];
        index += 1;
    }
    diff == 0
}
