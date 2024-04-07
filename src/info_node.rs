use std::rc::Rc;

use yew::{
    classes, function_component, html, use_state, Callback, Html, Properties, UseStateHandle,
};

use crate::{context_menu::ContextMenuItemProps, pill::Pill};

use log_viewer::{
    level_filter::LevelFilter,
    proto::log_level::{LogLevel, LogLevelLabel},
    state::{EventType, State},
};

#[derive(Clone, PartialEq, Properties)]
pub struct InfoNodeProps {
    pub state: Rc<State>,
    pub node_index: usize,
    pub level_filter: UseStateHandle<LevelFilter>,
}

#[function_component(InfoNode)]
pub fn info_node(props: &InfoNodeProps) -> Html {
    let node = &props.state.nodes[props.node_index];
    let event = node.index.map(|i| &props.state.events[i]);
    let expanded = use_state(|| node.expanded);

    let onclick = {
        let expanded = expanded.clone();
        move |_| {
            expanded.set(!*expanded);
        }
    };

    let body = || {
        html! {{
            node.children.iter().map(|child| {
                match child {
                    EventType::Message(message) => {
                        let event = &props.state.events[*message];
                        let message = &event.fields.message;
                        let level = event.level;
                        let target = &event.target;
                        let hidden = !props.level_filter.show(Some(target.clone()), &level);
                        let targets = &target.split("::").collect::<Vec<_>>();
                        let level_filter = props.level_filter.clone();

                        fn nest(level_filter: UseStateHandle<LevelFilter>, level: LogLevel, targets: &str, target: &str) -> Html {
                            let context_menu = vec![
                                ContextMenuItemProps {
                                    callback: {
                                        let level_filter = level_filter.clone();
                                        let targets = targets.to_string();
                                        Callback::from(move |_| {
                                            level_filter
                                                .set((*level_filter)
                                                .clone()
                                                .set_level(None, LogLevel::None)
                                                .set_level(Some(targets.to_string()), level));
                                        })
                                    },
                                    title: format!("Only show {targets}"),
                                },
                                ContextMenuItemProps {
                                    callback: {
                                        let level_filter = level_filter;
                                        let targets = targets.to_string();
                                        Callback::from(move |_| {
                                        level_filter.set((*level_filter).clone().set_level(Some(targets.to_string()), LogLevel::None))
                                    })},
                                    title: format!("Don't show {targets}"),
                                },
                            ];
                            let classes = classes!["p-1", "-m-1", "rounded-md", "hover:bg-gray-200"];

                            html!{ <Pill {context_menu} {classes}>{target}</Pill> }
                        }

                        html! {
                            if !hidden {
                                <div key={props.node_index} class={classes!["pl-6", "py-1", "m-1", "flex", "cursor-default", "select-none", "block"]}>
                                    <LogLevelLabel {level} />
                                    <span class={classes!["p-1", "px-2", "rounded-lg", format!("bg-{}", level.color())]}>
                                    {for targets.iter().enumerate().scan(String::new(), |state, (i, target)| {
                                        if i == 0 {
                                            state.push_str(target);
                                        } else {
                                            *state = format!("{state}::{target}");
                                        }
                                        Some((i, state.clone(), target))}).map(|(i, ts, t)| { html!{<>
                                            {if i != 0 {
                                                html!{{"::"}}
                                            } else {
                                                html!{}
                                            }}
                                            { nest(level_filter.clone(), level, &ts, t) }
                                        </>}})
                                    }
                                    </span>
                                    <pre>
                                    {message}
                                    </pre>
                                </div>
                            }
                        }

                    }
                    EventType::Node(node_index) => html! {
                        <InfoNode
                            key={*node_index}
                            state={props.state.clone()}
                            node_index={node_index}
                            level_filter={props.level_filter.clone()}
                        />
                    },
                }
            }).collect::<Html>()
        }}
    };

    if let Some(event) = event {
        let span_title = event.span.as_ref().unwrap().name.clone();
        let level = event.level;
        let target = &event.target;

        html! {
            <div class="flex w-full py-1">
                <div class="flex flex-col">

                    if *expanded {
                        <svg xmlns="http://www.w3.org/2000/svg" {onclick} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", "block" )}>
                            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
                        </svg>
                        <div class={classes!("grow", "w-3", "border-r", "border-black", "block")}></div>

                    } else {
                        <svg xmlns="http://www.w3.org/2000/svg" onclick={onclick.clone()} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", "block" )}>
                            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                        </svg>
                    }

                </div>
                <div>
                    <LogLevelLabel {level} />
                    <span class={classes!["m-1","p-1", "rounded-md", "bg-gray-200"]}>{target}</span>
                    {span_title}
                    if *expanded {
                        <span class={classes!["pt-1", "block"]}>{body()}</span>
                    }
                </div>
            </div>
        }
    } else {
        body()
    }
}
