use web_sys::MouseEvent;
use yew::{classes, function_component, html, use_context, Children, Html, Properties};

use crate::context_menu::{ContextMenuAction, ContextMenuContext, ContextMenuItemProps};

#[derive(Clone, PartialEq, Properties)]
pub struct PillProps {
    pub color: String,
    #[prop_or_default]
    pub children: Children,
    pub context_menu: Option<Vec<ContextMenuItemProps>>,
}

#[function_component(Pill)]
pub fn pill(props: &PillProps) -> Html {
    let context_menu = use_context::<ContextMenuContext>().unwrap();
    let items = props.context_menu.clone().unwrap_or_default();

    let oncontextmenu = {
        move |event: MouseEvent| {
            event.prevent_default();
            event.cancel_bubble();
            context_menu.dispatch(ContextMenuAction::Show(
                event.client_x(),
                event.client_y(),
                items.clone(),
            ))
        }
    };

    html! {<span
            class={classes!["m-1","p-1", "rounded-md", format!("bg-{}", props.color)]}
            {oncontextmenu}
        >
            {for props.children.iter()}
    </span>}
}
