use yew::prelude::*;

use web_sys::MouseEvent;

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
                if let Some(x) = self.translate.x.checked_sub(self.last_move.x - coords.x) {
                    self.translate.x = x;
                }

                if let Some(y) = self.translate.y.checked_sub(self.last_move.y - coords.y) {
                    self.translate.y = y;
                }

                self.last_move.x = coords.x;
                self.last_move.y = coords.y;
            }
            Message::MouseDown(coords) => {
                self.last_move = coords;
                self.is_mouse_down = true;
            }
            Message::MouseUp => self.is_mouse_down = false,
            Message::ZoomIn(amount) => self.scale += amount,
            Message::ZoomOut(amount) => self.scale -= amount,
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
                    <i class="fa-solid fa-lock-open"></i>
                </button>
            }
        } else {
            html! {
                <button class="button" onclick={on_lock} title="Lock">
                    <i class="fa-solid fa-lock"></i>
                </button>
            }
        };

        html! {
            <div class="movable-boxed" onmousedown={on_mouse_down} onmouseup={on_mouse_up} onmousemove={on_mouse_move} style={cursor}>
                <div class="movable-boxed-buttons">
                    <button class="button" onclick={on_reposition} title="Reposition">
                        <i class="fa-solid fa-arrows-to-dot"></i>
                    </button>
                    <button class="button" onclick={on_zoom_in} title="Zoom In">
                        <i class="fa-solid fa-plus"></i>
                    </button>
                    <button class="button" onclick={on_zoom_out} title="Zoom Out">
                        <i class="fa-solid fa-minus"></i>
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
