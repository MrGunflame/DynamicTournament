use yew::prelude::*;

use web_sys::MouseEvent;

use crate::components::button::Button;

pub struct MovableBoxed {
    translate: Coordinates,
    last_move: Coordinates,
    is_mouse_down: bool,
    scale: u32,

    is_locked: bool,
}

impl Component for MovableBoxed {
    type Message = Message;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            translate: Coordinates::default(),
            last_move: Coordinates::default(),
            is_mouse_down: false,
            scale: 100,

            is_locked: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::MoveAbsoluate(coords) => {
                self.translate = coords;
            }
            Message::Move(coords) => {
                self.translate.x = self
                    .translate
                    .x
                    .saturating_sub(self.last_move.x.saturating_sub(coords.x));

                self.translate.y = self
                    .translate
                    .y
                    .saturating_sub(self.last_move.y.saturating_sub(coords.y));

                self.last_move.x = coords.x;
                self.last_move.y = coords.y;
            }
            Message::MouseDown(coords) => {
                self.last_move = coords;
                self.is_mouse_down = true;
            }
            Message::MouseUp => self.is_mouse_down = false,
            Message::ZoomIn(amount) => self.scale = self.scale.saturating_add(amount),
            Message::ZoomOut(amount) => self.scale = self.scale.saturating_sub(amount),
            Message::ToggleLock => self.is_locked = !self.is_locked,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_locked = self.is_locked;

        let on_mouse_up = ctx.link().callback(|_: MouseEvent| Message::MouseUp);

        let on_mouse_down = ctx.link().batch_callback(move |e: MouseEvent| {
            if is_locked {
                return None;
            }

            e.prevent_default();

            Some(Message::MouseDown(Coordinates {
                x: e.client_x(),
                y: e.client_y(),
            }))
        });

        let is_mouse_down = self.is_mouse_down;
        let on_mouse_move = ctx.link().batch_callback(move |e: MouseEvent| {
            if is_locked {
                return None;
            }

            if is_mouse_down {
                e.prevent_default();

                Some(Message::Move(Coordinates {
                    x: e.client_x(),
                    y: e.client_y(),
                }))
            } else {
                None
            }
        });

        let on_reposition = ctx
            .link()
            .callback(|_| Message::MoveAbsoluate(Coordinates::default()));

        let on_zoom_in = ctx.link().callback(|_| Message::ZoomIn(5));

        let on_zoom_out = ctx.link().callback(|_| Message::ZoomOut(5));

        let on_lock = ctx.link().callback(|_| Message::ToggleLock);

        let cursor = if self.is_locked {
            "cursor: unset;"
        } else if self.is_mouse_down {
            "cursor: grabbing;"
        } else {
            "cursor: grab;"
        };

        let style = format!(
            "transform: translate({}px, {}px) scale({}%);",
            self.translate.x, self.translate.y, self.scale
        );

        let lock_button = if is_locked {
            html! {
                <button class="button" onclick={on_lock} title="Unlock">
                    <i aria-hidden="true" class="fa-solid fa-lock-open"></i>
                    <span class="sr-only">{ "Unlock" }</span>
                </button>
            }
        } else {
            html! {
                <button class="button" onclick={on_lock} title="Lock">
                    <i aria-hidden="true" class="fa-solid fa-lock"></i>
                    <span class="sr-only">{ "Lock" }</span>
                </button>
            }
        };

        html! {
            <div class="movable-boxed" onmousedown={on_mouse_down} onmouseup={on_mouse_up} onmousemove={on_mouse_move} style={cursor}>
                <div class="movable-boxed-buttons">
                    <Button onclick={on_reposition} title="Reposition">
                        <i aria-hidden="true" class="fa-solid fa-arrows-to-dot"></i>
                        <span class="sr-only">{ "Reposition" }</span>
                    </Button>
                    <button class="button" onclick={on_zoom_in} title="Zoom In">
                        <i aria-hidden="true" class="fa-solid fa-plus"></i>
                        <span class="sr-only">{ "Zoom In" }</span>
                    </button>
                    <button class="button" onclick={on_zoom_out} title="Zoom Out">
                        <i aria-hidden="true" class="fa-solid fa-minus"></i>
                        <span class="sr-only">{ "Zoom Out" }</span>
                    </button>
                    {lock_button}
                </div>
                <div class="movable-boxed-content" style={style}>
                    { for ctx.props().children.iter() }
                </div>
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub children: Children,
}

#[derive(Clone, Debug)]
pub enum Message {
    MoveAbsoluate(Coordinates),
    Move(Coordinates),
    MouseUp,
    MouseDown(Coordinates),
    ZoomIn(u32),
    ZoomOut(u32),
    ToggleLock,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Coordinates {
    x: i32,
    y: i32,
}
