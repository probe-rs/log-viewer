use yew::{classes, function_component, html, Callback, Html, Properties};

use crate::{level_filter::LevelFilter, proto::log_level::LogLevel};

#[derive(Clone, PartialEq, Properties)]
pub struct LevelPickerProps {
    pub(crate) level_filter: LevelFilter,
    pub(crate) on_select: Callback<LevelFilter>,
}

#[function_component(LevelPicker)]
pub fn level_picker(props: &LevelPickerProps) -> Html {
    let onselect = |level| {
        let level_filter = props.level_filter.clone();
        let on_select = props.on_select.clone();
        move |_| {
            let level_filter = level_filter.clone();
            let level_filter = if level_filter.show("", &level) {
                level_filter.remove_level(Some(""), &level)
            } else {
                level_filter.add_level(Some(""), level)
            };
            on_select.emit(level_filter)
        }
    };

    html! {
        [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ].into_iter().map(|level| {
            let checked = props.level_filter.show("", &level);
            let color = level.color();
            html!{<button onclick={onselect(level)} class={classes!["ml-3","my-3", "px-2", "py-1", "border", "border-black",format!("bg-{color}")]}>
                <label class="pr-2">{format!("{level}")}</label>
                <input type="checkbox" {checked} />
            </button>}
        }).collect::<Html>()
    }
}
