use std::fmt::{self, Display, Formatter};

use dynamic_tournament_macros::load_asset;
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
            Self::ExtraSmall => "dt-icon-size-xs",
            Self::Small => "dt-icon-size-sm",
            Self::Normal => "dt-icon-size-nl",
            Self::Large => "dt-icon-size-lg",
            Self::ExtraLarge => "dt-icon-size-xl",
            Self::ExtraLarge2 => "dt-icon-size-xl2",
        }
    }
}

impl Display for FaSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! fa_icon {
    ($($id:ident, $src:expr),*$(,)?) => {
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
                    let classes = format!("dt-icon {} {}", ctx.props().style, ctx.props().size);

                    let label = ctx.props().label;

                    html! {
                        <>
                            <img src={$src} alt={label} class={classes} />
                        </>
                    }
                }
            }
        )*
    };
}

fa_icon! {
    FaXmark, load_asset!("/icons/fontawesome/xmark.svg"),
    FaPen, load_asset!("/icons/fontawesome/pen.svg"),
    FaPenToSquare, load_asset!("/icons/fontawesome/pen-to-square.svg"),
    FaRotateLeft, load_asset!("/icons/fontawesome/rotate-left.svg"),
    FaTrash, load_asset!("/icons/fontawesome/trash.svg"),
    FaPlus, load_asset!("/icons/fontawesome/plus.svg"),
    FaMinus, load_asset!("/icons/fontawesome/minus.svg"),
    FaAngleLeft, load_asset!("/icons/fontawesome/angle-left.svg"),
    FaCompress, load_asset!("/icons/fontawesome/compress.svg"),
    FaLock, load_asset!("/icons/fontawesome/lock.svg"),
    FaLockOpen, load_asset!("/icons/fontawesome/lock-open.svg"),
}
