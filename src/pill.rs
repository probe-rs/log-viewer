use web_sys::MouseEvent;
use yew::{function_component, html, use_context, Children, Classes, Html, Properties};

use crate::context_menu::{ContextMenuAction, ContextMenuContext, ContextMenuItemProps};

#[derive(Clone, PartialEq, Properties)]
pub struct PillProps {
    #[prop_or_default]
    pub children: Children,
    pub context_menu: Option<Vec<ContextMenuItemProps>>,
    pub classes: Option<Classes>,
}

#[function_component(Pill)]
pub fn pill(props: &PillProps) -> Html {
    let context_menu = use_context::<ContextMenuContext>().unwrap();
    let items = props.context_menu.clone().unwrap_or_default();

    let oncontextmenu = {
        move |event: MouseEvent| {
            event.prevent_default();
            event.set_cancel_bubble(true);
            context_menu.dispatch(ContextMenuAction::Show(
                event.page_x(),
                event.page_y(),
                items.clone(),
            ))
        }
    };

    html! {<span
            class={&props.classes}
            {oncontextmenu}
        >
            {for props.children.iter()}
    </span>}
}
