use std::rc::Rc;

use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, MouseEvent};
use yew::{
    function_component, html, use_context, use_effect_with, use_reducer, use_state, Callback,
    Children, ContextProvider, Html, Properties, Reducible, UseReducerHandle,
};

#[function_component(ContextMenu)]
pub fn context_menu() -> Html {
    let context_menu = use_context::<ContextMenuContext>().unwrap();
    let listener = use_state(|| None);

    {
        let context_menu = context_menu.clone();
        use_effect_with((), move |_| {
            listener.set(Some(EventListener::new(
                &web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .dyn_into::<EventTarget>()
                    .unwrap(),
                "click",
                move |_event| context_menu.dispatch(ContextMenuAction::Hide),
            )));
        });
    }

    let show = &context_menu.show;

    if let Some((x, y, items)) = show {
        html! {<ul class="border border-black rounded-md bg-white" style={format!("position: absolute; top: {y}px; left: {x}px")}>
            {for items.iter().map(|i| html!{<ContextMenuItem callback={i.callback.clone()} title={i.title.clone()} />})}
        </ul>}
    } else {
        html! {<></>}
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ContextMenuItemProps {
    pub callback: Callback<()>,
    pub title: String,
}

#[function_component(ContextMenuItem)]
pub fn context_menu_item(props: &ContextMenuItemProps) -> Html {
    let onclick = {
        let callback = props.callback.clone();
        move |event: MouseEvent| {
            event.prevent_default();
            event.set_cancel_bubble(true);
            callback.emit(())
        }
    };

    html! {
        <li {onclick} class="m-1 px-2 cursor-default rounded-sm hover:bg-blue-500" style="min-width: 10em;">{&props.title}</li>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ContextMenuState {
    pub show: Option<(i32, i32, Vec<ContextMenuItemProps>)>,
}

impl Reducible for ContextMenuState {
    type Action = ContextMenuAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ContextMenuAction::Show(x, y, items) => ContextMenuState {
                show: Some((x, y, items)),
            }
            .into(),
            ContextMenuAction::Hide => ContextMenuState { show: None }.into(),
        }
    }
}

pub type ContextMenuContext = UseReducerHandle<ContextMenuState>;

#[derive(Properties, Debug, PartialEq)]
pub struct ContextMenuProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn ContextMenuProvider(props: &ContextMenuProviderProps) -> Html {
    let msg = use_reducer(|| ContextMenuState { show: None });

    html! {
        <ContextProvider<ContextMenuContext> context={msg}>
            {props.children.clone()}
        </ContextProvider<ContextMenuContext>>
    }
}

pub enum ContextMenuAction {
    Show(i32, i32, Vec<ContextMenuItemProps>),
    Hide,
}
