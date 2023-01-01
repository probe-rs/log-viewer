use web_sys::MouseEvent;
use yew::{classes, function_component, html, use_state, Callback, Html, Properties};

use crate::{level_filter::LevelFilter, proto::log_level::LogLevel};

#[derive(Clone, PartialEq, Properties)]
pub struct LevelPickerProps {
    pub(crate) level_filter: LevelFilter,
    pub(crate) on_select: Callback<LevelFilter>,
}

#[function_component(LevelPicker)]
pub fn level_picker(props: &LevelPickerProps) -> Html {
    let open = use_state(|| None);
    let onselect = |target: Option<String>, level| {
        let level_filter = props.level_filter.clone();
        let on_select = props.on_select.clone();
        move |_| {
            let level_filter = level_filter.clone();
            on_select.emit(level_filter.set_level(target.clone(), level))
        }
    };

    html! {<div class="flex">
        { for props.level_filter.matrix().iter().map(|(target, level)| {
            let color = level.color();
            let open = open.clone();
            let target_string = target.clone().unwrap_or_else(|| "default".into());
            let onopen = {
                let open = open.clone();
                let target = target.clone();
                move |_| {
                    let open = open.clone();
                    let target = target.clone();
                    if open.is_some() {
                        if *open != Some(target.clone()) {
                            open.set(Some(target))
                        } else {
                            open.set(None);
                        }
                    } else {
                        open.set(Some(target))
                    }
                }
            };
            let onremove = {
                let level_filter = props.level_filter.clone();
                let on_select = props.on_select.clone();
                let target = target.clone();
                move |event: MouseEvent| {
                    event.set_cancel_bubble(true);
                    event.prevent_default();
                    on_select.emit(level_filter.clone().remove(&target));
                }
            };
            let target = target.clone();
            let open = (*open).clone();
            html!{<div class="flex">
                <button onclick={onopen} class={classes!["flex", "ml-3","my-3", "px-2", "py-1", "border", "border-black", format!("bg-{color}")]}>
                    <span>{target_string}</span>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" class="flex pl-1 pt-1 w-5 h-5 cursor-pointer" onclick={onremove}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
                <ul class={classes!(if open == Some(target.clone()) { "block" } else { "hidden" })}>
                {
                    [
                        LogLevel::None,
                        LogLevel::Error,
                        LogLevel::Warn,
                        LogLevel::Info,
                        LogLevel::Debug,
                        LogLevel::Trace,
                    ].into_iter().map(|level| {
                        let checked = props.level_filter.matrix().get(&target) == Some(&level);
                        let color = level.color();
                        html!{<button onclick={onselect(target.clone(), level)} class={classes!["my-3", "px-2", "py-1", "border-r", "border-t", "border-b", "border-black", format!("bg-{color}")]}>
                            <label class="pr-2">{format!("{level}")}</label>
                            <input type="checkbox" {checked} />
                        </button>}
                    }).collect::<Html>()
                }
                </ul>
            </div>}
        }) }
    </div>}
}
