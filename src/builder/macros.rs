//! Macros for ergonomic state machine construction.

/// Generate State trait implementation for simple enums.
///
/// # Example
///
/// ```
/// use mindset::state_enum;
///
/// state_enum! {
///     pub enum WorkflowState {
///         Start,
///         Processing,
///         Done,
///         Failed,
///     }
///     final: [Done, Failed]
///     error: [Failed]
/// }
/// ```
#[macro_export]
macro_rules! state_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),* $(,)?
        }

        $(final: [$($final:ident),* $(,)?])?
        $(error: [$($error:ident),* $(,)?])?
    ) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }

        impl $crate::core::State for $name {
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }

            fn is_final(&self) -> bool {
                match self {
                    $($(Self::$final => true,)*)?
                    _ => false,
                }
            }

            fn is_error(&self) -> bool {
                match self {
                    $($(Self::$error => true,)*)?
                    _ => false,
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::core::State;

    state_enum! {
        enum TestState {
            Initial,
            Processing,
            Complete,
            Failed,
        }
        final: [Complete, Failed]
        error: [Failed]
    }

    #[test]
    fn state_enum_macro_generates_trait() {
        let state = TestState::Initial;
        assert_eq!(state.name(), "Initial");
        assert!(!state.is_final());
        assert!(!state.is_error());

        let complete = TestState::Complete;
        assert!(complete.is_final());
        assert!(!complete.is_error());

        let failed = TestState::Failed;
        assert!(failed.is_final());
        assert!(failed.is_error());
    }

    #[test]
    fn state_enum_supports_visibility() {
        // The macro should work with pub visibility
        state_enum! {
            pub enum PublicState {
                A,
                B,
            }
            final: [B]
        }

        let _state = PublicState::A;
    }

    #[test]
    fn state_enum_works_without_final_error() {
        state_enum! {
            enum MinimalState {
                One,
                Two,
            }
        }

        let state = MinimalState::One;
        assert!(!state.is_final());
        assert!(!state.is_error());
    }
}
