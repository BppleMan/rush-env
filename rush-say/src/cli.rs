pub mod border {
    use clap::ValueEnum;
    use rush_say::{BorderStyle, border_factory};
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;

    macro_rules! border_type {
        ($($name:ident, $con:expr $(,$ignore:expr)+);+ $(;)?) => {
            $($name,)+
        };
    }
    // macro_rules! border_type {
    //     ($($name:ident, $con:expr $(,$ignore:expr)+);+ $(;)?) => {
    //         #[allow(non_camel_case_types)]
    //         #[derive(Debug, Clone, Copy, ValueEnum)]
    //         pub enum BorderType {
    //             $($name,)+
    //         }
    //
    //         impl BorderType {
    //             pub fn build(&self) -> BorderStyle {
    //                 match self {
    //                     $(Self::$name => BorderStyle::$name()),+
    //                 }
    //             }
    //         }
    //
    //         impl Display for BorderType {
    //             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    //                 match self {
    //                     $(Self::$name => write!(f, "{}", $con),)+
    //                 }
    //             }
    //         }
    //
    //         impl FromStr for BorderType {
    //             type Err = String;
    //
    //             fn from_str(s: &str) -> Result<Self, Self::Err> {
    //                 match s {
    //                     $($con => Ok(Self::$name),)+
    //                     _ => Err(format!("Unknown border type {s}")),
    //                 }
    //             }
    //         }
    //     };
    // }
    //
    // border_factory!(border_type);
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy, ValueEnum)]
    pub enum BorderType {
        border_factory!(border_type);
    }

    #[allow(clippy::derivable_impls)]
    impl Default for BorderType {
        fn default() -> Self {
            BorderType::simple
        }
    }
}

pub mod comment {
    use clap::ValueEnum;
    use rush_say::{CommentStyle, comment_factory};
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;

    macro_rules! comment_type {
        ($($name:ident, $con:expr $(,$ignore:expr)+);+ $(;)?) => {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, ValueEnum)]
            pub enum CommentType {
                $($name,)+
            }

            impl CommentType {
                pub fn build(&self) -> CommentStyle {
                    match self {
                        $(Self::$name => CommentStyle::$name()),+
                    }
                }
            }

            impl Display for CommentType {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$name => write!(f, "{}", $con),)+
                    }
                }
            }

            impl FromStr for CommentType {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $($con => Ok(Self::$name),)+
                        _ => Err(format!("Unknown comment type {s}")),
                    }
                }
            }
        };
    }

    comment_factory!(comment_type);

    #[allow(clippy::derivable_impls)]
    impl Default for CommentType {
        fn default() -> Self {
            CommentType::shell
        }
    }
}
