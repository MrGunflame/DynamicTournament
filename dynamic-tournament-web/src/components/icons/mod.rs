use std::fmt::{self, Display, Formatter};

use yew::{html, Component, Context, Html, Properties};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub label: &'static str,
    #[prop_or_default]
    pub style: FaStyle,
    #[prop_or_default]
    pub size: FaSize,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FaStyle {
    #[default]
    Solid,
}

impl FaStyle {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Solid => "fa-solid",
        }
    }
}

impl Display for FaStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FaSize {
    ExtraSmall,
    Small,
    #[default]
    Normal,
    Large,
    ExtraLarge,
    ExtraLarge2,
}

impl FaSize {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ExtraSmall => "fa-xs",
            Self::Small => "fa-sm",
            Self::Normal => "",
            Self::Large => "fa-lg",
            Self::ExtraLarge => "fa-xl",
            Self::ExtraLarge2 => "fa-2xl",
        }
    }
}

impl Display for FaSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! fa_icon {
    ($($id:ident, $name:expr),*$(,)?) => {
        $(
            #[derive(Debug)]
            pub struct $id;

            impl Component for $id {
                type Message = ();
                type Properties = Props;

                fn create(_ctx: &Context<Self>) -> Self {
                    Self
                }

                fn view(&self, ctx: &Context<Self>) -> Html {
                    let classes = format!("{} {} {}", $name, ctx.props().style, ctx.props().size);

                    let label = ctx.props().label;

                    html! {
                        <>
                            <i aria-hidden="true" class={classes}></i>
                            <span class="sr-only">{ label }</span>
                        </>
                    }
                }
            }
        )*
    };
}

fa_icon! {
    FaXmark, "fa-xmark",
    FaPen, "fa-pen",
    FaPenToSquare, "fa-pen-to-square",
    FaRotateLeft, "fa-rotate-left",
    FaTrash, "fa-trash",
    FaPlus, "fa-plus",
}
