use std::fmt;

/// Errors returned by parsing, conversion, palette, and contrast operations.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorError {
    /// A textual color representation is malformed or unsupported.
    InvalidSyntax {
        /// Original input.
        input: String,
        /// Human-readable explanation.
        reason: String,
    },
    /// A CSS named color was not recognized.
    UnknownColorName(String),
    /// A numeric component is outside its accepted finite range.
    OutOfRange {
        /// Component or option name.
        component: &'static str,
        /// Supplied value.
        value: f64,
        /// Inclusive lower bound.
        min: f64,
        /// Inclusive upper bound.
        max: f64,
    },
    /// A strict conversion produced a color outside the sRGB gamut.
    OutOfGamut,
    /// An operation requires one or more values.
    EmptyInput(&'static str),
    /// A count-like option is too small.
    InvalidCount {
        /// Option name.
        name: &'static str,
        /// Supplied count.
        value: usize,
        /// Smallest accepted count.
        minimum: usize,
    },
    /// Transparency requires an explicit opaque canvas for contrast evaluation.
    AlphaRequiresCanvas,
    /// A requested contrast target is outside `[1, 21]`.
    InvalidContrastTarget(f64),
    /// No sRGB foreground can meet the requested contrast.
    UnreachableContrast {
        /// Requested ratio.
        target: f64,
        /// Best available ratio.
        maximum: f64,
    },
    /// A generic parameter has an invalid domain.
    InvalidParameter {
        /// Parameter name.
        name: &'static str,
        /// Expected domain.
        reason: &'static str,
    },
}

impl ColorError {
    pub(crate) fn syntax(input: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidSyntax {
            input: input.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSyntax { input, reason } => {
                write!(f, "invalid color syntax `{input}`: {reason}")
            }
            Self::UnknownColorName(name) => write!(f, "unknown CSS color name `{name}`"),
            Self::OutOfRange {
                component,
                value,
                min,
                max,
            } => write!(
                f,
                "component `{component}` is {value}, expected a finite value in [{min}, {max}]"
            ),
            Self::OutOfGamut => write!(f, "color is outside the sRGB gamut"),
            Self::EmptyInput(name) => write!(f, "`{name}` must not be empty"),
            Self::InvalidCount {
                name,
                value,
                minimum,
            } => write!(
                f,
                "`{name}` is {value}, expected an integer greater than or equal to {minimum}"
            ),
            Self::AlphaRequiresCanvas => write!(
                f,
                "transparent colors require an explicit opaque canvas before contrast is measured"
            ),
            Self::InvalidContrastTarget(value) => write!(
                f,
                "contrast target is {value}, expected a finite ratio in [1, 21]"
            ),
            Self::UnreachableContrast { target, maximum } => write!(
                f,
                "contrast target {target}:1 is unreachable; best available ratio is {maximum}:1"
            ),
            Self::InvalidParameter { name, reason } => {
                write!(f, "invalid parameter `{name}`: {reason}")
            }
        }
    }
}

impl std::error::Error for ColorError {}
