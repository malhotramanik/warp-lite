use warp_cli::agent::Harness;
use warp_graphql::managed_secrets::ManagedSecretType;

pub struct AuthSecretTypeField {
    pub label: &'static str,
    /// Placeholder text shown inside the input editor. Falls back to `label` if `None`.
    pub placeholder: Option<&'static str>,
    pub optional: bool,
    /// Whether the field contains sensitive data and should be password-masked.
    pub sensitive: bool,
}

pub struct AuthSecretTypeInfo {
    pub display_name: &'static str,
    pub secret_type: ManagedSecretType,
    pub fields: &'static [AuthSecretTypeField],
    pub learn_more_url: &'static str,
}

pub fn auth_secret_types_for_harness(harness: Harness) -> &'static [AuthSecretTypeInfo] {
    match harness {
        Harness::Claude => &CLAUDE_AUTH_SECRET_TYPES,
        Harness::Codex => &CODEX_AUTH_SECRET_TYPES,
        Harness::Oz | Harness::OpenCode | Harness::Gemini | Harness::Pi | Harness::Unknown => &[],
    }
}


const CODEX_LEARN_MORE_URL: &str =
    "https://docs.warp.dev/platform/harnesses/authentication/#connecting-codex-credentials";
const CLAUDE_LEARN_MORE_URL: &str =
    "https://docs.warp.dev/platform/harnesses/authentication/#connecting-claude-code-credentials";


static CODEX_AUTH_SECRET_TYPES: [AuthSecretTypeInfo; 1] = [AuthSecretTypeInfo {
    display_name: "OpenAI API Key",
    secret_type: ManagedSecretType::OpenaiApiKey,
    learn_more_url: CODEX_LEARN_MORE_URL,
    fields: &[
        AuthSecretTypeField {
            label: "OPENAI_API_KEY",
            placeholder: Some("sk-..."),
            optional: false,
            sensitive: true,
        },
        AuthSecretTypeField {
            label: "BASE_URL",
            placeholder: Some("https://us.api.openai.com/v1"),
            optional: true,
            sensitive: false,
        },
    ],
}];

static CLAUDE_AUTH_SECRET_TYPES: [AuthSecretTypeInfo; 3] = [
    AuthSecretTypeInfo {
        display_name: "Anthropic API Key",
        secret_type: ManagedSecretType::AnthropicApiKey,
        learn_more_url: CLAUDE_LEARN_MORE_URL,
        fields: &[AuthSecretTypeField {
            label: "ANTHROPIC_API_KEY",
            placeholder: Some("sk-ant-..."),
            optional: false,
            sensitive: true,
        }],
    },
    AuthSecretTypeInfo {
        display_name: "Bedrock API Key",
        secret_type: ManagedSecretType::AnthropicBedrockApiKey,
        learn_more_url: CLAUDE_LEARN_MORE_URL,
        fields: &[
            AuthSecretTypeField {
                label: "AWS_BEARER_TOKEN_BEDROCK",
                placeholder: Some("Bearer token"),
                optional: false,
                sensitive: true,
            },
            AuthSecretTypeField {
                label: "AWS_REGION",
                placeholder: Some("us-east-1"),
                optional: false,
                sensitive: false,
            },
        ],
    },
    AuthSecretTypeInfo {
        display_name: "Bedrock Access Key",
        secret_type: ManagedSecretType::AnthropicBedrockAccessKey,
        learn_more_url: CLAUDE_LEARN_MORE_URL,
        fields: &[
            AuthSecretTypeField {
                label: "AWS_ACCESS_KEY_ID",
                placeholder: Some("AKIA..."),
                optional: false,
                sensitive: true,
            },
            AuthSecretTypeField {
                label: "AWS_SECRET_ACCESS_KEY",
                placeholder: Some("Secret access key"),
                optional: false,
                sensitive: true,
            },
            AuthSecretTypeField {
                label: "AWS_SESSION_TOKEN",
                placeholder: Some("Session token (temporary credentials only)"),
                optional: true,
                sensitive: true,
            },
            AuthSecretTypeField {
                label: "AWS_REGION",
                placeholder: Some("us-east-1"),
                optional: false,
                sensitive: false,
            },
        ],
    },
];
